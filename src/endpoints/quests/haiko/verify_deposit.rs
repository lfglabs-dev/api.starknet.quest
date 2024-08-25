use std::sync::Arc;

use crate::utils::fetch_json_from_url;
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
use serde_json::json;

#[route(get, "/quests/haiko/verify_deposit")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 142;
    let addr = &query.addr;

    let url = format!(
        "{}&user={}",
        state.conf.quests.haiko.api_endpoint,
        to_hex(*addr)
    );

    match fetch_json_from_url(url).await {
        Ok(response) => {
            // check if user has deposited his funds
            let response_data = response
                .get("data")
                .unwrap()
                .get("result")
                .unwrap()
                .as_bool()
                .unwrap();
            return if response_data {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Funds not deposited".to_string())
            };
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
