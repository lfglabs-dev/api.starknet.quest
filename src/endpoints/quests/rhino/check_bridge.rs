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
use serde_json::json;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 81;
    let addr = &query.addr;

    // create get request to rhino api for verification
    let url = format!(
        "{}/?address={}",
        state.conf.rhino.api_endpoint,
        to_hex(*addr)
    );

    match fetch_json_from_url(url).await {
        Ok(response) => {
            /*
              API response is in the format:
                    {
                        "result": true/false
                    }
            */
            // check if user has bridged his funds
            let has_bridged = response.get("result").unwrap().as_bool().unwrap();
            return if has_bridged {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Funds not bridged".to_string())
            };
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
