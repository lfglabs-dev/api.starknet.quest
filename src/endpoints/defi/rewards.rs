use std::collections::HashMap;
use std::sync::Arc;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};

use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    models::{AppState, Call, Claim, EkuboRewards, NimboraRewards, NostraResponse, ZkLendReward},
    utils::get_error,
};
use starknet::core::types::FieldElement;

// Define constants for token and contract addresses
const STRK_TOKEN: &str = "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
const STRK_DECIMALS: u8 = 18;
const STRK_NAME: &str = "STRK";
const NIMBORA_CLAIM_CONTRACT: &str =
    "0x07ed46700bd12bb1ee8a33a8594791003f9710a1ab18edd958aed86a8f82d3d1";
const NOSTRA_CLAIM_CONTRACT: &str =
    "0x008faa2edc6833a6ad0625f1128d56cf471c3f9649ff2201d9ef49d7e9bb18de";

#[derive(Serialize, Deserialize, Debug)]
pub enum RewardSource {
    ZkLend,
    Nostra,
    Nimbora,
    Ekubo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonReward {
    pub amount: String,
    pub proof: Vec<String>,
    pub recipient: Option<String>,
    pub reward_id: Option<u64>,
    pub claim_contract: String,
    pub token_address: Option<String>,
    pub token_decimals: Option<u8>,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub reward_source: RewardSource,
    pub claimed: bool,
}

#[route(get, "/defi/rewards")]
pub async fn get_defi_rewards(
    State(_): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let addr = match params.get("addr") {
        Some(address) => address,
        None => return get_error("Missing 'addr' parameter".to_string()),
    };

    // Validate the address format
    if FieldElement::from_hex_be(addr).is_err() {
        return get_error("Invalid address format".to_string()).into_response();
    }

    // Retry up to 3 times with increasing intervals between attempts.
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        .with(TracingMiddleware::default())
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let (zklend_rewards, nostra_rewards, nimbora_rewards, ekubo_rewards) = tokio::join!(
        fetch_zklend_rewards(&client, addr),
        fetch_nostra_rewards(&client, addr),
        fetch_nimbora_rewards(&client, addr),
        fetch_ekubo_rewards(&client, addr),
    );

    // Unwrap results or provide defaults in case of errors
    let zklend_rewards = zklend_rewards.unwrap_or_default();
    let nostra_rewards = nostra_rewards.unwrap_or_default();
    let nimbora_rewards = nimbora_rewards.unwrap_or_default();
    let ekubo_rewards = ekubo_rewards.unwrap_or_default();
    
    // Create Call Data
    let zklend_calls = create_calls(&zklend_rewards, addr);
    let nostra_calls = create_calls(&nostra_rewards, addr);
    let nimbora_calls = create_calls(&nimbora_rewards, addr);
    let ekubo_calls = create_calls(&ekubo_rewards, addr);

    let response_data = json!({
        "rewards": {
            "zklend": zklend_rewards,
            "nostra": nostra_rewards,
            "nimbora": nimbora_rewards,
            "ekubo": ekubo_rewards
        },
        "calls": {
            "zklend": zklend_calls,
            "nostra": nostra_calls,
            "nimbora": nimbora_calls,
            "ekubo": ekubo_calls
        }
    });

    (StatusCode::OK, Json(response_data)).into_response()
}

async fn fetch_zklend_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
) -> Result<Vec<CommonReward>, Error> {
    let zklend_url = format!("https://app.zklend.com/api/reward/all/{}", addr);
    let response = client.get(&zklend_url)
        .headers(get_headers())
        .send().await?;

    match response.json::<Vec<ZkLendReward>>().await {
        Ok(result) => {
            let rewards = result
                .into_iter()
                .map(|reward| CommonReward {
                    amount: reward.amount.value,
                    proof: reward.proof,
                    recipient: Some(reward.recipient),
                    reward_id: Some(reward.claim_id),
                    claim_contract: reward.claim_contract,
                    token_address: None,
                    token_decimals: Some(reward.token.decimals),
                    token_name: Some(reward.token.name),
                    token_symbol: Some(reward.token.symbol),
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
    match response.json::<NostraResponse>().await {
        Ok(result) => {
            let rewards = result
                .documents
                .into_iter()
                .map(|doc| CommonReward {
                    amount: doc.reward,
                    proof: doc.proofs,
                    recipient: Some(doc.account),
                    reward_id: None,
                    claim_contract: NOSTRA_CLAIM_CONTRACT.to_string(),
                    token_address: Some(STRK_TOKEN.to_string()),
                    token_decimals: Some(STRK_DECIMALS),
                    token_name: Some(STRK_NAME.to_string()),
                    token_symbol: Some(STRK_NAME.to_string()),
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
) -> Result<Vec<CommonReward>, Error> {
    let nimbora_url = format!(
        "https://strk-dist-backend.nimbora.io/get_calldata?address={}",
        addr
    );

    let response = client.get(&nimbora_url)
        .headers(get_headers())
        .send().await?;

    match response.json::<NimboraRewards>().await {
        Ok(result) => {
            let reward = CommonReward {
                amount: result.amount,
                proof: result.proof,
                recipient: None,
                reward_id: None,
                token_address: Some(STRK_TOKEN.to_string()),
                token_decimals: Some(STRK_DECIMALS),
                token_name: Some(STRK_NAME.to_string()),
                token_symbol: Some(STRK_NAME.to_string()),
                claim_contract: NIMBORA_CLAIM_CONTRACT.to_string(),
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
) -> Result<Vec<CommonReward>, Error> {
    let ekubo_url = format!(
        "https://mainnet-api.ekubo.org/airdrops/{}?token={}",
        addr, STRK_TOKEN
    );
    let response = client.get(&ekubo_url)
        .headers(get_headers())
        .send().await?;

    match response.json::<Vec<EkuboRewards>>().await {
        Ok(result) => {
            let rewards = result
                .into_iter()
                .map(|reward| CommonReward {
                    amount: reward.claim.amount,
                    proof: reward.proof,
                    recipient: Some(reward.claim.claimee),
                    reward_id: Some(reward.claim.id),
                    claim_contract: reward.contract_address,
                    token_address: Some(STRK_TOKEN.to_string()),
                    token_decimals: Some(STRK_DECIMALS),
                    token_name: Some(STRK_NAME.to_string()),
                    token_symbol: Some(STRK_NAME.to_string()),
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

fn create_calls(rewards: &[CommonReward], addr: &str) -> Vec<Call> {
    rewards
        .iter()
        .filter(|reward| !reward.claimed)
        .map(|reward| {
            let call_data: Vec<String> = match reward.reward_source {
                RewardSource::Nimbora => vec![
                    reward.amount.clone(),
                    serde_json::to_string(&reward.proof).unwrap_or_default(),
                ],
                RewardSource::Nostra => vec![
                    reward.amount.clone(),
                    serde_json::to_string(&reward.proof).unwrap_or_default(),
                ],
                RewardSource::ZkLend => vec![
                    serde_json::to_string(&vec![Claim{
                        id: reward.reward_id.unwrap(),
                        amount: reward.amount.clone(),
                        claimee: addr.to_string()
                    }]).unwrap_or_default(),
                    serde_json::to_string(&reward.proof).unwrap_or_default(),
                ],
                RewardSource::Ekubo => vec![
                    serde_json::to_string(&vec![Claim{
                        id: reward.reward_id.unwrap(),
                        amount: reward.amount.clone(),
                        claimee: addr.to_string()
                    }]).unwrap_or_default(),
                    serde_json::to_string(&reward.proof).unwrap_or_default(),
                ],
            };
            Call {
                entry_point: "claim".to_string(),
                contract: reward.claim_contract.clone(),
                call_data,
                regex: "claim".to_string(),
            }
        })
        .collect()
}

fn get_headers()-> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"),
    );
    headers
}