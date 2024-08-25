use std::sync::Arc;

use crate::models::VerifyQuery;
use crate::utils::{make_api_request, to_hex, CompletedTasksTrait};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use serde_json::json;

#[route(get, "/quests/rango/check_trade")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 92;
    let mut address_hex = to_hex(query.addr);

    // remove "0x"
    address_hex.remove(0);
    address_hex.remove(0);

    // remove leading zeroes
    while address_hex.starts_with("0") {
        address_hex.remove(0);
    }

    // add "0x" back
    address_hex.insert(0, 'x');
    address_hex.insert(0, '0');

    let res = make_api_request(
        &state.conf.rango.api_endpoint,
        &address_hex,
        Some(&state.conf.rango.api_key),
    )
    .await;

    match res {
        true => {
            return match state.upsert_completed_task(query.addr, task_id).await {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            };
        }
        false => get_error("User has not completed the task".to_string()),
    }
}
