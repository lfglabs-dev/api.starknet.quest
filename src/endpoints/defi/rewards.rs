use crate::{
    models::{
        AppState, CommonReward, ContractCall, DefiReward, EkuboRewards, NimboraRewards,
        NostraPeriodsResponse, NostraResponse, RewardSource, VesuRewards, ZkLendReward,
    },
    utils::{check_if_unclaimed, read_contract, to_hex, to_hex_trimmed},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::stream::{FuturesOrdered, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::{core::types::FieldElement, macros::selector};
use std::{str::FromStr, sync::Arc, vec};

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardQuery {
    addr: FieldElement,
}

#[route(get, "/defi/rewards")]
pub async fn get_defi_rewards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RewardQuery>,
) -> impl IntoResponse {
    let addr = to_hex(query.addr);

    // Retry up to 3 times with increasing intervals between attempts.
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let (zklend_rewards, nostra_rewards, nimbora_rewards, ekubo_rewards, vesu_rewards) = tokio::join!(
        fetch_zklend_rewards(&client, &addr, &state),
        fetch_nostra_rewards(&client, &addr, &state),
        fetch_nimbora_rewards(&client, &addr, &state),
        fetch_ekubo_rewards(&client, &addr, &state),
        fetch_vesu_rewards(&client, &addr, &state),
    );

    let zklend_rewards = zklend_rewards.unwrap_or_default();
    let nostra_rewards = nostra_rewards.unwrap_or_default();
    let nimbora_rewards = nimbora_rewards.unwrap_or_default();
    let ekubo_rewards = ekubo_rewards.unwrap_or_default();
    let vesu_rewards = vesu_rewards.unwrap_or_default();

    let all_rewards = [
        &zklend_rewards,
        &nostra_rewards,
        &nimbora_rewards,
        &ekubo_rewards,
        &vesu_rewards,
    ];

    let all_calls: Vec<ContractCall> = all_rewards
        .iter()
        .flat_map(|rewards| create_calls(rewards, &addr))
        .collect();

    let response_data = json!({
        "rewards": {
            "zklend": extract_rewards(&zklend_rewards),
            "nostra": extract_rewards(&nostra_rewards),
            "nimbora": extract_rewards(&nimbora_rewards),
            "ekubo": extract_rewards(&ekubo_rewards),
            "vesu": extract_rewards(&vesu_rewards)
        },
        "calls": all_calls
    });

    (StatusCode::OK, Json(response_data)).into_response()
}

async fn fetch_zklend_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    state: &AppState,
) -> Result<Vec<CommonReward>, Error> {
    let logger = &state.logger;
    let zklend_url = format!("https://app.zklend.com/api/reward/all/{}", addr);
    let response = client
        .get(&zklend_url)
        .headers(get_headers())
        .send()
        .await?;

    match response.json::<Vec<ZkLendReward>>().await {
        Ok(result) => {
            let rewards = result
                .into_iter()
                .filter(|reward| !reward.claimed)
                .map(|reward| CommonReward {
                    amount: reward.amount.value,
                    displayed_amount: reward.amount.value,
                    proof: reward.proof,
                    reward_id: Some(reward.claim_id),
                    claim_contract: reward.claim_contract,
                    token_symbol: reward.token.symbol,
                    reward_source: RewardSource::ZkLend,
                    claimed: reward.claimed,
                    start_date: None,
                    end_date: None,
                })
                .collect();
            Ok(rewards)
        }
        Err(err) => {
            logger.warning(format!("Failed to deserialize zkLend response: {:?}", err));
            Err(Error::Reqwest(err))
        }
    }
}

async fn fetch_nostra_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    state: &AppState,
) -> Result<Vec<CommonReward>, Error> {
    let logger = &state.logger;
    let url =
        "https://us-east-2.aws.data.mongodb-api.com/app/data-yqlpb/endpoint/data/v1/action/find";

    let proof_request_body = json!({
        "dataSource": "nostra-production",
        "database": "prod-a-nostra-db",
        "collection": "rewardProofs",
        "filter": { "account": addr }
    });

    let periods_request_body = json!({
        "dataSource": "nostra-production",
        "database": "prod-a-nostra-db",
        "collection": "rewardPeriods"
    });

    let (periods_resp, rewards_resp) = tokio::try_join!(
        client
            .post(url)
            .headers(get_headers())
            .json(&periods_request_body)
            .send(),
        client
            .post(url)
            .headers(get_headers())
            .json(&proof_request_body)
            .send()
    )?;

    let reward_periods = match periods_resp.json::<NostraPeriodsResponse>().await {
        Ok(result) => result,
        Err(err) => {
            logger.warning(format!(
                "Failed to deserialize Nostra periods response: {:?}",
                err
            ));
            NostraPeriodsResponse { documents: vec![] }
        }
    };

    let rewards = match rewards_resp.json::<NostraResponse>().await {
        Ok(result) => result,
        Err(err) => {
            logger.warning(format!(
                "Failed to deserialize Nostra rewards response: {:?}",
                err
            ));
            return Err(Error::Reqwest(err));
        }
    };

    let addr_field = FieldElement::from_hex_be(addr).unwrap();
    let tasks: FuturesOrdered<_> = rewards
        .documents
        .into_iter()
        .rev()
        .map(|doc| {
            let addr_field = addr_field.clone();
            let token_symbol = state.conf.tokens.strk.symbol.clone();
            let matching_period = reward_periods
                .documents
                .iter()
                .find(|period| period.id == doc.reward_id && period.defi_spring_rewards);

            async move {
                if let Some(distributor) =
                    matching_period.and_then(|period| period.defi_spring_rewards_distributor)
                {
                    if check_if_unclaimed(
                        state,
                        distributor,
                        selector!("amount_already_claimed"),
                        vec![addr_field],
                        RewardSource::Nostra,
                    )
                    .await
                    {
                        Some(CommonReward {
                            amount: doc.reward,
                            displayed_amount: doc.reward,
                            proof: doc.proofs,
                            reward_id: None,
                            claim_contract: distributor,
                            token_symbol,
                            reward_source: RewardSource::Nostra,
                            claimed: false,
                            start_date: None,
                            end_date: None,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .collect();
    let active_rewards = tasks.filter_map(|res| async move { res }).collect().await;
    Ok(active_rewards)
}

// Fetch rewards from nimbora
async fn fetch_nimbora_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    state: &AppState,
) -> Result<Vec<CommonReward>, Error> {
    let logger = &state.logger;
    let config = &state.conf;
    let nimbora_url = format!(
        "https://strk-dist-backend.nimbora.io/get_calldata?address={}",
        addr
    );
    let response = client
        .get(&nimbora_url)
        .headers(get_headers())
        .send()
        .await?;

    let strk_symbol = config.tokens.strk.symbol.clone();

    match response.json::<NimboraRewards>().await {
        Ok(result) => {
            let amount = result.amount;
            let claimed_amount = read_contract(
                state,
                config.rewards.nimbora.contract,
                selector!("amount_already_claimed"),
                vec![FieldElement::from_str(addr).unwrap()],
            )
            .await
            .unwrap()[0];
            if claimed_amount == amount {
                return Ok(vec![]);
            }
            let reward = CommonReward {
                amount: amount,
                displayed_amount: amount - claimed_amount,
                proof: result.proof,
                reward_id: None,
                token_symbol: strk_symbol.clone(),
                claim_contract: config.rewards.nimbora.contract,
                reward_source: RewardSource::Nimbora,
                claimed: false,
                start_date: None,
                end_date: None,
            };
            Ok(vec![reward])
        }
        Err(err) => {
            logger.warning(format!("Failed to deserialize nimbora response: {:?}", err));
            Err(Error::Reqwest(err))
        }
    }
}

async fn fetch_ekubo_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    state: &AppState,
) -> Result<Vec<CommonReward>, Error> {
    let logger = &state.logger;
    let strk_token = state.conf.tokens.strk.clone();
    let ekubo_url = format!(
        "https://mainnet-api.ekubo.org/airdrops/{}?token={}",
        addr,
        to_hex(strk_token.contract)
    );

    let response = client.get(&ekubo_url).headers(get_headers()).send().await?;

    let rewards = match response.json::<Vec<EkuboRewards>>().await {
        Ok(result) => result,
        Err(err) => {
            logger.warning(format!(
                "Failed to deserialize Ekubo rewards response: {:?}",
                err
            ));
            return Err(Error::Reqwest(err));
        }
    };
    let (last_reward_id, last_reward_amount) = rewards
        .last()
        .map(|reward| (reward.claim.id, reward.claim.amount))
        .unzip();
    let tasks: FuturesOrdered<_> = rewards
        .into_iter()
        .rev()
        .map(|reward| {
            let strk_token = strk_token.clone();
            async move {
                if check_if_unclaimed(
                    state,
                    reward.contract_address,
                    selector!("is_claimed"),
                    vec![FieldElement::from(reward.claim.id)],
                    RewardSource::Ekubo,
                )
                .await
                {
                    Some(CommonReward {
                        amount: reward.claim.amount,
                        displayed_amount: reward.claim.amount,
                        proof: reward.proof,
                        reward_id: Some(reward.claim.id),
                        claim_contract: reward.contract_address,
                        token_symbol: strk_token.symbol,
                        reward_source: RewardSource::Ekubo,
                        claimed: false,
                        start_date: Some(reward.start_date),
                        end_date: Some(reward.end_date),
                    })
                } else {
                    None
                }
            }
        })
        .collect();
    let active_rewards: Vec<CommonReward> =
        tasks.filter_map(|res| async move { res }).collect().await;
    // We check if the latest reward has already been claimed
    if active_rewards.len() >= 1
        && (active_rewards[0].reward_id.unwrap() != last_reward_id.unwrap()
            || active_rewards[0].amount != last_reward_amount.unwrap())
    {
        return Ok(vec![]);
    }
    // If several tasks have both the same start and end date, only the last one should returned
    let filtered_tasks =
        active_rewards
            .into_iter()
            .fold(Vec::<CommonReward>::new(), |mut acc, reward| {
                if !acc
                    .iter()
                    .any(|r| r.start_date == reward.start_date && r.end_date == reward.end_date)
                {
                    acc.push(reward);
                }
                acc
            });

    Ok(filtered_tasks)
}

async fn fetch_vesu_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    state: &AppState,
) -> Result<Vec<CommonReward>, Error> {
    let logger = &state.logger;
    let vesu_url = format!("https://api.vesu.xyz/users/{}/strk-rewards", addr);
    let response = client.get(&vesu_url).headers(get_headers()).send().await?;

    match response.json::<VesuRewards>().await {
        Ok(result) => {
            let call_data = match result.data.distributor_data.call_data {
                Some(data) => data,
                None => {
                    return Ok(vec![]);
                }
            };

            let strk_token = state.conf.tokens.strk.clone();
            let config = &state.conf;

            let disctributed_amount: FieldElement = result
                .data
                .distributor_data
                .distributed_amount
                .parse()
                .expect("Failed to parse string to integer");

            let claimed_amount: FieldElement = result
                .data
                .distributor_data
                .claimed_amount
                .parse()
                .expect("Failed to parse string to integer");

            let amount = disctributed_amount - claimed_amount;

            // If amount is 0, return empty vector
            if amount == FieldElement::ZERO {
                return Ok(vec![]);
            }

            let reward = CommonReward {
                amount: disctributed_amount,
                displayed_amount: amount,
                proof: call_data.proof,
                reward_id: None,
                claim_contract: config.rewards.vesu.contract,
                token_symbol: strk_token.symbol,
                reward_source: RewardSource::Vesu,
                claimed: false,
                start_date: None,
                end_date: None,
            };
            Ok(vec![reward])
        }
        Err(err) => {
            logger.warning(format!("Failed to deserialize vesu response: {:?}", err));
            // Err(Error::Reqwest(err))
            Ok(vec![]) // Return empty vector instead of error
        }
    }
}

fn create_calls(rewards: &[CommonReward], addr: &str) -> Vec<ContractCall> {
    rewards
        .iter()
        .filter(|reward| !reward.claimed)
        .map(|reward| {
            let calldata: Vec<String> = match reward.reward_source {
                RewardSource::ZkLend | RewardSource::Ekubo => {
                    let mut data = vec![
                        to_hex_trimmed(FieldElement::from(reward.reward_id.unwrap())),
                        addr.to_string(),
                        to_hex_trimmed(reward.amount),
                        to_hex_trimmed(FieldElement::from(reward.proof.len())),
                    ];
                    data.extend(reward.proof.clone());
                    data
                }
                RewardSource::Nimbora | RewardSource::Nostra | RewardSource::Vesu => {
                    let mut data = vec![
                        to_hex_trimmed(reward.amount),
                        to_hex_trimmed(FieldElement::from(reward.proof.len())),
                    ];
                    data.extend(reward.proof.clone());
                    data
                }
            };

            ContractCall {
                contractaddress: to_hex(reward.claim_contract),
                calldata,
                entrypoint: "claim".to_string(),
            }
        })
        .collect()
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
        ),
    );
    headers
}

fn extract_rewards(common_rewards: &[CommonReward]) -> Vec<DefiReward> {
    common_rewards
        .iter()
        .map(|reward| DefiReward {
            amount: reward.displayed_amount,
            token_symbol: reward.token_symbol.clone(),
        })
        .collect()
}
