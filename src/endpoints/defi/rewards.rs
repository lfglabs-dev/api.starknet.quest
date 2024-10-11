use crate::{
    config::Config,
    models::{
        AppState, CommonReward, ContractCall, DefiReward, EkuboRewards, NimboraRewards,
        NostraResponse, RewardSource, ZkLendReward,
    },
    utils::to_hex,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::core::types::FieldElement;
use std::sync::Arc;

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
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let (zklend_rewards, nostra_rewards, nimbora_rewards, ekubo_rewards) = tokio::join!(
        fetch_zklend_rewards(&client, &addr),
        fetch_nostra_rewards(&client, &addr, &state.conf),
        fetch_nimbora_rewards(&client, &addr, &state.conf),
        fetch_ekubo_rewards(&client, &addr, &state.conf),
    );

    let zklend_rewards = zklend_rewards.unwrap_or_default();
    let nostra_rewards = nostra_rewards.unwrap_or_default();
    let nimbora_rewards = nimbora_rewards.unwrap_or_default();
    let ekubo_rewards = ekubo_rewards.unwrap_or_default();

    let zklend_calls = create_calls(&zklend_rewards, &addr);
    let nostra_calls = create_calls(&nostra_rewards, &addr);
    let nimbora_calls = create_calls(&nimbora_rewards, &addr);
    let ekubo_calls = create_calls(&ekubo_rewards, &addr);

    let all_calls: Vec<ContractCall> =
        [zklend_calls, nostra_calls, nimbora_calls, ekubo_calls].concat();

    let response_data = json!({
        "rewards": {
            "zklend": extract_rewards(&zklend_rewards),
            "nostra": extract_rewards(&nostra_rewards),
            "nimbora": extract_rewards(&nimbora_rewards),
            "ekubo": extract_rewards(&ekubo_rewards)
        },
        "calls": all_calls
    });

    (StatusCode::OK, Json(response_data)).into_response()
}

async fn fetch_zklend_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
) -> Result<Vec<CommonReward>, Error> {
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
                .map(|reward| CommonReward {
                    amount: reward.amount.value,
                    proof: reward.proof,
                    reward_id: Some(reward.claim_id),
                    claim_contract: reward.claim_contract,
                    token_symbol: reward.token.symbol,
                    reward_source: RewardSource::ZkLend,
                    claimed: reward.claimed,
                })
                .collect();
            Ok(rewards)
        }
        Err(err) => {
            eprintln!("Failed to deserialize zkLend response: {:?}", err);
            Err(Error::Reqwest(err))
        }
    }
}

// Fetch rewards from Nostra
async fn fetch_nostra_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    config: &Config,
) -> Result<Vec<CommonReward>, Error> {
    let nostra_request_body = json!({
        "dataSource": "nostra-production",
        "database": "prod-a-nostra-db",
        "collection": "rewardProofs",
        "filter": { "account": addr }
    });
    let response = client.post("https://us-east-2.aws.data.mongodb-api.com/app/data-yqlpb/endpoint/data/v1/action/find")
        .headers(get_headers())
        .json(&nostra_request_body)
        .send()
        .await?;

    let nostra_claim_contract = to_hex(config.rewards.nostra.contract);
    let strk_symbol = config.tokens.strk.symbol.clone();
    match response.json::<NostraResponse>().await {
        Ok(result) => {
            let rewards = result
                .documents
                .into_iter()
                .map(|doc| CommonReward {
                    amount: doc.reward,
                    proof: doc.proofs,
                    reward_id: None,
                    claim_contract: nostra_claim_contract.clone(),
                    token_symbol: strk_symbol.clone(),
                    reward_source: RewardSource::Nostra,
                    claimed: false,
                })
                .collect();
            Ok(rewards)
        }
        Err(err) => {
            eprintln!("Failed to deserialize Nostra response: {:?}", err);
            Err(Error::Reqwest(err))
        }
    }
}

// Fetch rewards from nimbora
async fn fetch_nimbora_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    config: &Config,
) -> Result<Vec<CommonReward>, Error> {
    let nimbora_url = format!(
        "https://strk-dist-backend.nimbora.io/get_calldata?address={}",
        addr
    );

    let response = client
        .get(&nimbora_url)
        .headers(get_headers())
        .send()
        .await?;

    let nimbora_claim_contract = to_hex(config.rewards.nimbora.contract);
    let strk_symbol = config.tokens.strk.symbol.clone();

    match response.json::<NimboraRewards>().await {
        Ok(result) => {
            let reward = CommonReward {
                amount: result.amount,
                proof: result.proof,
                reward_id: None,
                token_symbol: strk_symbol.clone(),
                claim_contract: nimbora_claim_contract.clone(),
                reward_source: RewardSource::Nimbora,
                claimed: false,
            };
            Ok(vec![reward])
        }
        Err(err) => {
            eprintln!("Failed to deserialize nimbora response: {:?}", err);
            Err(Error::Reqwest(err))
        }
    }
}

async fn fetch_ekubo_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
    config: &Config,
) -> Result<Vec<CommonReward>, Error> {
    let strk_token = config.tokens.strk.clone();
    let ekubo_url = format!(
        "https://mainnet-api.ekubo.org/airdrops/{}?token={}",
        addr,
        to_hex(strk_token.contract)
    );

    let response = client.get(&ekubo_url).headers(get_headers()).send().await?;

    match response.json::<Vec<EkuboRewards>>().await {
        Ok(result) => {
            let rewards = result
                .into_iter()
                .map(|reward| CommonReward {
                    amount: reward.claim.amount,
                    proof: reward.proof,
                    reward_id: Some(reward.claim.id),
                    claim_contract: reward.contract_address,
                    token_symbol: strk_token.symbol.clone(),
                    reward_source: RewardSource::Ekubo,
                    claimed: false,
                })
                .collect();
            Ok(rewards)
        }
        Err(err) => {
            eprintln!("Failed to deserialize ekubo response: {:?}", err);
            Err(Error::Reqwest(err))
        }
    }
}

fn create_calls(rewards: &[CommonReward], addr: &str) -> Vec<ContractCall> {
    rewards
        .iter()
        // .filter(|reward| !reward.claimed)
        .map(|reward| {
            let call_data: Vec<String> = match reward.reward_source {
                RewardSource::ZkLend | RewardSource::Ekubo => {
                    let mut data = vec![
                        to_hex(FieldElement::from(reward.reward_id.unwrap())),
                        addr.to_string(),
                        to_hex(reward.amount),
                        to_hex(FieldElement::from(reward.proof.len())),
                    ];
                    data.extend(reward.proof.clone());
                    data
                }
                RewardSource::Nimbora | RewardSource::Nostra => {
                    let mut data = vec![
                        to_hex(reward.amount),
                        to_hex(FieldElement::from(reward.proof.len())),
                    ];
                    data.extend(reward.proof.clone());
                    data
                }
            };

            ContractCall {
                contract: reward.claim_contract.clone(),
                call_data,
                entry_point: "claim".to_string(),
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
            amount: reward.amount.clone(),
            token_symbol: reward.token_symbol.clone(),
            claimed: reward.claimed,
        })
        .collect()
}
