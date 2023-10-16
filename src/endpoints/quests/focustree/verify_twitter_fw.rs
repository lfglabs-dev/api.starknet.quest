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
    let task_id = 62;
    match state.upsert_completed_task(query.addr, task_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
        Err(e) => get_error(format!("{}", e)),
    }
}
