use std::sync::Arc;

use crate::models::VerifyQuery;
use crate::utils::{to_hex, CompletedTasksTrait};
use crate::{models::AppState, utils::get_error};
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
    let task_id = 115;
    let address_hex = to_hex(query.addr);

    // make get request to pyramid api for verification
    let url = format!(
        "{}/has-deposited-starknet?starknet_address={}",
        state.conf.brine.api_endpoint,
        to_hex(query.addr)
    );

    let res = make_brine_request(url.as_str(), &state.conf.brine.api_key, &address_hex).await;

    let response = match res {
        Ok(response) => response,
        Err(_) => return get_error(format!("Try again later")),
    };

    if let Some(_) = response.get("status") {
        let data = response.get("payload").unwrap();
        if let Some(res) = data.get("has_deposited") {
            if res.as_bool().unwrap() {
                return match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                };
            }
        }
    }
    get_error("Not yet deposited".to_string())
}

async fn make_brine_request(
    endpoint: &str,
    api_key: &str,
    addr: &str,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    match client
        .get(endpoint)
        .header("X-API-Key", api_key)
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => Ok(json),
            Err(_) => Err(format!("Funds not deposited")),
        },
        Err(_) => Err(format!("Funds not bridged")),
    }
}
