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
use axum_auto_routes::route;
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;

#[derive(Debug, Deserialize)]
pub struct ElementResponse {
    #[allow(dead_code)]
    code: u32,
    data: bool,
}

#[route(get, "/quests/element/element/verify_is_eligible")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 65;
    if query.addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let url = format!(
        "https://api.element.market/openapi/v1/qualify/check?address={}&taskId=100120231026231111",
        to_hex(query.addr)
    );
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("accept", "application/json")
        .header("x-api-key", state.conf.quests.element.api_key.clone())
        .send()
        .await
    {
        Ok(response) => match response.text().await {
            Ok(text) => match serde_json::from_str::<ElementResponse>(&text) {
                Ok(res) => {
                    if res.data {
                        match state.upsert_completed_task(query.addr, task_id).await {
                            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                            Err(e) => get_error(format!("{}", e)),
                        }
                    } else {
                        get_error("You have not interacted with Element.".to_string())
                    }
                }
                Err(e) => get_error(format!(
                    "Failed to deserialize result from Element API: {} for response: {}",
                    e, text
                )),
            },
            Err(e) => get_error(format!(
                "Failed to get JSON response while fetching Element API: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch Element API: {}", e)),
    }
}
