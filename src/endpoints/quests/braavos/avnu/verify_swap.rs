use std::sync::Arc;

use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 85;
    let hex_addr = format!("{:#x}", query.addr);

    // Fetch AVNU endpoint to get user score
    let url = format!("https://starknet.api.avnu.fi/v1/quest/takers/{}", hex_addr);
    let client = reqwest::Client::new();
    let response_result = client.get(url).send().await;
    let response = match response_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => json,
                Err(e) => {
                    return get_error(format!(
                        "Failed to get JSON response while fetching user info: {}",
                        e
                    ));
                }
            }
        }
        Err(e) => {
            return get_error(format!("Failed to send request to fetch user info: {}", e));
        }
    };
    let score = match response["volumeInUSD"].as_f64() {
        Some(s) => s,
        None => {
            return get_error("Failed to get user info from response data".to_string());
        }
    };

    if score == 0.0 {
        get_error("You have not made a swap on AVNU yet.".to_string())
    } else {
        match state.upsert_completed_task(query.addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(e) => get_error(format!("{}", e)),
        }
    }
}
