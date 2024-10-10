use std::sync::Arc;
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{
    models::{AppState, ZkLendReward, NostraResponse, EkuboRewards, NimboraRewards, Call},
    utils::get_error,
};
use starknet::core::types::FieldElement;

// Define constants for token and contract addresses
const EKUBO_TOKEN: &str = "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
const NIMBORA_CLAIM_CONTRACT: &str = "0x07ed46700bd12bb1ee8a33a8594791003f9710a1ab18edd958aed86a8f82d3d1";
const NOSTRA_CLAIM_CONTRACT: &str = "0x008faa2edc6833a6ad0625f1128d56cf471c3f9649ff2201d9ef49d7e9bb18de";

// Define a struct to capture common reward data across all protocols
#[derive(Serialize, Deserialize)]
pub struct CommonReward {
    pub amount: String,
    pub proof: Vec<String>,
    pub recipient: Option<String>,
    pub reward_id: Option<String>,
    pub claim_contract: String,
    pub token_address: Option<String>,
    pub token_name: Option<String>,
    pub token_decimals: Option<u8>,
    pub reward_source: Option<String>,
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

    let client = Client::new();

    let zklend_rewards = match fetch_zklend_rewards(&client, addr).await {
        Ok(rewards) => rewards,
        Err(_) => vec![],
    };
    let nostra_rewards = match fetch_nostra_rewards(&client, addr).await {
        Ok(rewards) => rewards,
        Err(_) => vec![]
    };
    let nimbora_rewards = match fetch_nimbora_rewards(&client, addr).await {
        Ok(rewards) => rewards,
        Err(_) => vec![]
    };

    let ekubo_rewards = match fetch_ekubo_rewards(&client, addr).await {
        Ok(rewards) => rewards,
        Err(_) => vec![]
    };

    // Create Call Data
    let zklend_calls = create_calls(&zklend_rewards, addr);
    let nostra_calls = create_calls(&nostra_rewards, addr);
    let nimbora_calls = create_calls(&nimbora_rewards, addr);
    let ekubo_calls = create_calls(&ekubo_rewards, addr);

    let response_data = json!({
        "rewards": {
            "zklend_rewards": zklend_rewards,
            "nostra_rewards": nostra_rewards,
            "nimbora_rewards": nimbora_rewards,
            "ekubo_rewards": ekubo_rewards
        },
        "calls": {
            "zklend_calls": zklend_calls,
            "nostra_calls": nostra_calls,
            "nimbora_calls": nimbora_calls,
            "ekubo_calls": ekubo_calls
        }
    });

    (StatusCode::OK, Json(response_data)).into_response()
}


async fn fetch_zklend_rewards(client: &Client, addr: &str) -> Result<Vec<CommonReward>, reqwest::Error> {
    let zklend_url = format!("https://app.zklend.com/api/reward/all/{}", addr);
    let response = client.get(&zklend_url).send().await?;
    let rewards = response.json::<Vec<ZkLendReward>>().await?
        .into_iter().map(|reward| CommonReward {
            amount: reward.amount.value,
            proof: reward.proof,
            recipient: Some(reward.recipient),
            reward_id: Some(reward.claim_id.to_string()),
            claim_contract: reward.claim_contract,
            token_address: None,
            token_name: Some(reward.token.name),
            token_decimals: Some(reward.token.decimals),
            reward_source: Some("ZkLend".to_string()),
            claimed: reward.claimed
        }).collect();
    Ok(rewards)
}

// Fetch rewards from Nostra
async fn fetch_nostra_rewards(client: &Client, addr: &str) -> Result<Vec<CommonReward>, reqwest::Error> {
    let nostra_request_body = json!({
        "dataSource": "nostra-production",
        "database": "prod-a-nostra-db",
        "collection": "rewardProofs",
        "filter": { "account": addr.to_lowercase() }
    });
    let response = client.post("https://us-east-2.aws.data.mongodb-api.com/app/data-yqlpb/endpoint/data/v1/action/find")
        .json(&nostra_request_body)
        .send()
        .await?;
    let rewards = response.json::<NostraResponse>().await?
        .documents.into_iter().map(|doc| CommonReward {
            amount: doc.reward,
            proof: doc.proofs,
            recipient: Some(doc.account),
            reward_id: Some(doc.reward_id),
            claim_contract: NOSTRA_CLAIM_CONTRACT.to_string(),
            token_address: None,
            token_name: None,
            token_decimals: None,
            reward_source: Some(doc.reward_from),
            claimed: false
        }).collect();
    Ok(rewards)
}


// Fetch rewards from nimbora
async fn fetch_nimbora_rewards(client: &Client, addr: &str) -> Result<Vec<CommonReward>, reqwest::Error> {
    let nimbora_url = format!("https://strk-dist-backend.nimbora.io/get_calldata?address={}", addr);
    let response = client.get(&nimbora_url).send().await?;
    let data = response.json::<NimboraRewards>().await?;

    let reward = CommonReward {
        amount: data.amount,
        proof: data.proof,
        recipient: None,
        reward_id: None,
        token_address: None,
        token_name: None,
        token_decimals: None,
        claim_contract: NIMBORA_CLAIM_CONTRACT.to_string(),
        reward_source: Some("Nimbora".to_string()),
        claimed: false,
    };

    Ok(vec![reward])
}


async fn fetch_ekubo_rewards(client: &Client, addr: &str) -> Result<Vec<CommonReward>, reqwest::Error> {
    let ekubo_url = format!("https://mainnetapi.ekubo.org/airdrops/{}?token={}", addr, EKUBO_TOKEN);
    let response = client.get(&ekubo_url).send().await?;
    let rewards = response.json::<Vec<EkuboRewards>>().await?
        .into_iter().map(|reward| CommonReward {
            amount: reward.claim.amount,
            proof: reward.proof,
            recipient: Some(reward.claim.claimee),
            reward_id: Some(reward.claim.id.to_string()),
            claim_contract: reward.contract_address,
            token_name: None,
            token_decimals: None,
            token_address: Some(EKUBO_TOKEN.to_string()),
            reward_source: Some("Ekubo".to_string()),
            claimed: false
        }).collect();
    Ok(rewards)
}

fn create_calls(rewards: &Vec<CommonReward>, addr: &str) -> Vec<Call> {
    rewards.iter()
        .filter(|reward| !reward.claimed)
        .map(|reward| Call {
            entry_point: "claim".to_string(),
            contract: reward.claim_contract.clone(),
            call_data: vec![
                reward.reward_id.clone().unwrap_or_default(),
                addr.to_string(),
                reward.amount.clone(),
                serde_json::to_string(&reward.proof).unwrap_or_default(),
            ],
            regex: "claim".to_string(),
        })
        .collect()
}