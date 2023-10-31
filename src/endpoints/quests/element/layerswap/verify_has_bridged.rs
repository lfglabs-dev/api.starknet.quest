use std::sync::Arc;

use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error, to_hex, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct LayerswapResponse {
    data: Option<Vec<DataEntry>>,
    error: Option<LayerswapError>,
}

#[derive(Debug, Deserialize)]
struct DataEntry {
    status: String,
    created_date: String,
}

#[derive(Debug, Deserialize)]
struct LayerswapError {
    message: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 70;
    let url = format!(
        "https://bridge-api.layerswap.io/api/explorer/{}",
        to_hex(query.addr)
    );

    let three_months_ago = Utc::now() - Duration::days(90);
    let client = reqwest::Client::new();
    let response_result = client.get(url).send().await;
    match response_result {
        Ok(response) => match response.json::<LayerswapResponse>().await {
            Ok(res) => {
                if let Some(err) = &res.error {
                    return get_error(format!("Received error from Layerswap: {}", err.message));
                }

                // Check if there is data and if any entry has "completed" status & was made in the last 3 months
                if res.data.as_ref().unwrap_or(&vec![]).iter().any(|entry| {
                    entry.status == "completed"
                        && DateTime::parse_from_rfc3339(&entry.created_date)
                            .unwrap_or(Utc::now().into())
                            >= three_months_ago
                }) {
                    match state.upsert_completed_task(query.addr, task_id).await {
                        Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                        Err(e) => get_error(format!("{}", e)),
                    }
                } else {
                    get_error(
                        "You haven't bridge any ETH or USDC to Starknet using Layerswap."
                            .to_string(),
                    )
                }
            }
            Err(e) => get_error(format!(
                "Failed to get JSON response while fetching Layerswap data: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch Layerswap api: {}", e)),
    }
}
