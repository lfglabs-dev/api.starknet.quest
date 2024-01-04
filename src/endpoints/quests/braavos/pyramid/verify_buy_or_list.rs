use std::sync::Arc;

use crate::{
    models::{AppState},
    utils::{get_error},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::models::{VerifyQuery};
use crate::utils::{CompletedTasksTrait, fetch_json_from_url, to_hex};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 107;

    // make get request to focustree api for verification
    let url = format!(
        "{}/{}/isEligibleForQuest", state.conf.pyramid.api_endpoint,
        to_hex(query.addr)
    );
    
    match fetch_json_from_url(url).await {
        Ok(response) => {
            // check if user has sent NFT to focus tree wallet
            let has_sent_nft = response.get("result").unwrap().as_bool().unwrap();
            return if has_sent_nft {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => {
                        (StatusCode::OK, Json(json!({"res": true}))).into_response()
                    }
                    Err(e) => {
                        get_error(format!("{}", e))
                    }
                }
            } else {
                get_error("NFT not bought/listed".to_string())
            };
        }
        Err(e) => get_error(format!("{}", e)),
    }
}