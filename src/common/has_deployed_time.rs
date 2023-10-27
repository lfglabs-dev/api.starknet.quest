use std::sync::Arc;

use crate::utils::DeployedTimesTrait;
use crate::{
    models::{AppState, DeployedTime},
    utils::to_hex,
};
use mongodb::{bson::doc, Collection};
use starknet::core::types::FieldElement;

pub async fn execute_has_deployed_time(
    state: Arc<AppState>,
    addr: &FieldElement,
) -> Result<u32, String> {
    // Check if we have already a result in the db
    let deployed_times_collection: Collection<DeployedTime> = state.db.collection("deployed_times");
    let filter = doc! { "addr": addr.to_string() };
    if let Ok(Some(document)) = deployed_times_collection.find_one(filter, None).await {
        println!("Found deployed time in db: {:?}", document);
        return Ok(document.timestamp);
    }

    // If not we fetch it from the API and store it in the db
    let url = format!(
        "https://api.starkscan.co/api/v0/transactions?from_block=1&limit=1&contract_address={}&order_by=asc",
        to_hex(*addr)
    );
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("accept", "application/json")
        .header("x-api-key", state.conf.starkscan.api_key.clone())
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(timestamp) = json["data"][0]["timestamp"].as_i64() {
                    match state
                        .upsert_deployed_timestamp(*addr, timestamp as u32)
                        .await
                    {
                        Ok(_) => Ok(timestamp as u32),
                        Err(e) => Err(format!("{}", e)),
                    }
                } else {
                    Err("Wallet not deployed.".to_string())
                }
            }
            Err(e) => Err(format!(
                "Failed to get JSON response while fetching user transaction data: {}",
                e
            )),
        },
        Err(e) => Err(format!("Failed to fetch user transactions from API: {}", e)),
    }
}
