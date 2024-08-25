use std::sync::Arc;

use crate::models::AppState;
use crate::{common::has_deployed_time::execute_has_deployed_time, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::core::types::FieldElement;

#[derive(Debug, Serialize, Deserialize)]

pub struct GetDeployedTimeQuery {
    addr: FieldElement,
}

#[route(get, "/get_deployed_time")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetDeployedTimeQuery>,
) -> impl IntoResponse {
    match execute_has_deployed_time(state, &query.addr).await {
        Ok(timestamp) => (StatusCode::OK, Json(json!({ "timestamp": timestamp }))).into_response(),
        Err(e) => get_error(e),
    }
}
