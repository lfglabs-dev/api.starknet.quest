use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    models::{AppState, Call, EkuboRewards, NimboraRewards, NostraResponse, ZkLendReward},
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
    
#[derive(Serialize, Deserialize)]
pub struct CommonReward {
    pub amount: String,
    pub proof: Vec<String>,
    pub recipient: Option<String>,
    pub reward_id: Option<String>,
    pub claim_contract: String,
    pub token_address: Option<String>,
    pub token_decimals: Option<u8>,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub reward_source: String,
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

    let zklend_rewards = (fetch_zklend_rewards(&client, addr).await).unwrap_or_default();
    let nostra_rewards =(fetch_nostra_rewards(&client, addr).await).unwrap_or_default();
    let nimbora_rewards =(fetch_nimbora_rewards(&client, addr).await).unwrap_or_default();
    let ekubo_rewards = (fetch_ekubo_rewards(&client, addr).await).unwrap_or_default();

    // Create Call Data
    let zklend_calls = create_calls(&zklend_rewards);
    let nostra_calls = create_calls(&nostra_rewards);
    let nimbora_calls = create_calls(&nimbora_rewards);
    let ekubo_calls = create_calls(&ekubo_rewards);

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
) -> Result<Vec<CommonReward>, reqwest::Error> {
    let zklend_url = format!("https://app.zklend.com/api/reward/all/{}", addr);
    let response = client.get(&zklend_url).send().await.unwrap();
    let rewards = response
        .json::<Vec<ZkLendReward>>()
        .await?
        .into_iter()
        .map(|reward| CommonReward {
            amount: reward.amount.value,
            proof: reward.proof,
            recipient: Some(reward.recipient),
            reward_id: Some(reward.claim_id.to_string()),
            claim_contract: reward.claim_contract,
            token_address: None,
            token_decimals: Some(reward.token.decimals),
            token_name: Some(reward.token.name),
            token_symbol: Some(reward.token.symbol),
            reward_source: "ZkLend".to_string(),
            claimed: reward.claimed,
        })
        .collect();
    Ok(rewards)
}

// Fetch rewards from Nostra
async fn fetch_nostra_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
) -> Result<Vec<CommonReward>, reqwest::Error> {
    let nostra_request_body = json!({
        "dataSource": "nostra-production",
        "database": "prod-a-nostra-db",
        "collection": "rewardProofs",
        "filter": { "account": addr.to_lowercase() }
    });
    let response = client.post("https://us-east-2.aws.data.mongodb-api.com/app/data-yqlpb/endpoint/data/v1/action/find")
        .json(&nostra_request_body)
        .send()
        .await.unwrap();
    let rewards = response
        .json::<NostraResponse>()
        .await?
        .documents
        .into_iter()
        .map(|doc| CommonReward {
            amount: doc.reward,
            proof: doc.proofs,
            recipient: Some(doc.account),
            reward_id: Some(doc.reward_id),
            claim_contract: NOSTRA_CLAIM_CONTRACT.to_string(),
            token_address: Some(STRK_TOKEN.to_string()),
            token_decimals: Some(STRK_DECIMALS),
            token_name: Some(STRK_NAME.to_string()),
            token_symbol: Some(STRK_NAME.to_string()),
            reward_source: "Nostra".to_string(),
            claimed: false,
        })
        .collect();
    Ok(rewards)
}

// Fetch rewards from nimbora
async fn fetch_nimbora_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
) -> Result<Vec<CommonReward>, reqwest::Error> {
    let nimbora_url = format!(
        "https://strk-dist-backend.nimbora.io/get_calldata?address={}",
        addr
    );
    let response = client.get(&nimbora_url).send().await.unwrap();
    let data = response.json::<NimboraRewards>().await?;

    let reward = CommonReward {
        amount: data.amount,
        proof: data.proof,
        recipient: None,
        reward_id: None,
        token_address: Some(STRK_TOKEN.to_string()),
        token_decimals: Some(STRK_DECIMALS),
        token_name: Some(STRK_NAME.to_string()),
        token_symbol: Some(STRK_NAME.to_string()),
        claim_contract: NIMBORA_CLAIM_CONTRACT.to_string(),
        reward_source: "Nimbora".to_string(),
        claimed: false,
    };

    Ok(vec![reward])
}

async fn fetch_ekubo_rewards(
    client: &ClientWithMiddleware,
    addr: &str,
) -> Result<Vec<CommonReward>, reqwest::Error> {
    let ekubo_url = format!(
        "https://mainnetapi.ekubo.org/airdrops/{}?token={}",
        addr, STRK_TOKEN
    );
    let response = client.get(&ekubo_url).send().await.unwrap();
    let rewards = response
        .json::<Vec<EkuboRewards>>()
        .await?
        .into_iter()
        .map(|reward| CommonReward {
            amount: reward.claim.amount,
            proof: reward.proof,
            recipient: Some(reward.claim.claimee),
            reward_id: Some(reward.claim.id.to_string()),
            claim_contract: reward.contract_address,
            token_address: Some(STRK_TOKEN.to_string()),
            token_decimals: Some(STRK_DECIMALS),
            token_name: Some(STRK_NAME.to_string()),
            token_symbol: Some(STRK_NAME.to_string()),
            reward_source: "Ekubo".to_string(),
            claimed: false,
        })
        .collect();
    Ok(rewards)
}

fn create_calls(rewards: &[CommonReward]) -> Vec<Call> {
    rewards
        .iter()
        .filter(|reward| !reward.claimed)
        .map(|reward| {
            Call {
                entry_point: "claim".to_string(),
                contract: reward.claim_contract.clone(),
                call_data: vec![
                    reward.amount.clone(),
                    serde_json::to_string(&reward.proof).unwrap_or_default(), 
                ],
                regex: "claim".to_string(),
            }
        })
        .collect()
}
