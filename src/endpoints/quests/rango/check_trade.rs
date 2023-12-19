use std::sync::Arc;

use crate::models::VerifyQuery;
use crate::utils::{CompletedTasksTrait, to_hex};
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
    let task_id = 92;
    let address_hex = to_hex(query.addr);
    let res = make_rango_request(
        &state.conf.rango.api_endpoint,
        &state.conf.rango.api_key,
        address_hex,
    )
        .await;
    let response = match res {
        Ok(response) => response,
        Err(e) => return get_error(format!("{}", e)),
    };

    if let Some(_) = response.get("data") {
        if let Some(result) = response.get("result") {
            if result.as_bool().unwrap() {
                return match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                };
            }
        }
    }
    get_error("User has not completed the task".to_string())
}

async fn make_rango_request(
    endpoint: &str,
    api_key: &str,
    addr: String,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    match client
        .post(endpoint)
        .json(&json!({
            "address": addr,
        }))
        .header("apiKey", api_key)
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                let result = if let Some(data) = json.get("data") {
                    if let Some(result) = data.get("result") {
                        result.as_bool().unwrap()
                    } else {
                        false
                    }
                } else {
                    false
                };
                return match result {
                    true => Ok(json!({"res": true})),
                    false => Err(format!("Funds not bridged")),
                };
            }
            Err(_) => Err(format!("Funds not bridged")),
        },
        Err(_) => Err(format!("Funds not bridged")),
    }
}
