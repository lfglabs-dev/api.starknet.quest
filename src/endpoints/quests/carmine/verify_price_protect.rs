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
use axum_auto_routes::route;
use serde_json::json;
use starknet::core::types::FieldElement;

#[route(get, "/quests/carmine/verify_price_protect")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = query.task_id.unwrap();
    let addr = query.addr;

    let api_url = "https://api.carmine.finance/api/v1/mainnet/price-protect-users";

    // Check if the addr is in the "data" field of the API response
    let response = reqwest::get(api_url)
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let data = response["data"].as_array().unwrap();
    let mut found = false;
    for address in data {
        if FieldElement::from_hex_be(address.as_str().unwrap()).expect("Failed to parse address")
            == addr
        {
            found = true;
            break;
        }
    }
    if found {
        match state.upsert_completed_task(addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(e) => get_error(format!("{}", e)),
        }
    } else {
        get_error("You didn't open price protect for at least 10$ on Carmine.".to_string())
    }
}
