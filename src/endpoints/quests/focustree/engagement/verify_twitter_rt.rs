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

#[route(
    get,
    "/quests/focustree/verify_twitter_rw_user",
    crate::endpoints::quests::focustree::engagement::verify_twitter_rt
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 88;
    match state.upsert_completed_task(query.addr, task_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
        Err(e) => get_error(format!("{}", e)),
    }
}
