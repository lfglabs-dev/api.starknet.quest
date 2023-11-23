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
use crate::models::EmailQuery;
use crate::utils::{fetch_json_from_url};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EmailQuery>,
) -> impl IntoResponse {
    let task_id = 87;

    // add check for empty email
    if query.email == "" {
        get_error("Please enter your email".to_string());
    }

    // make get request to focustree api for verification
    let url = format!(
        "{}/{}", state.conf.quests.focustree.api_endpoint,
        query.email
    );

    match fetch_json_from_url(url).await {
        Ok(response) => {
            let error_message = response.get("message").unwrap().as_str().unwrap();

            if error_message.len() > 0 {
                return get_error("User not found".to_string());
            }
            let has_sent_nft = response.get("hasSentNFTToFocusTreeWallet").unwrap().as_bool().unwrap();
            if has_sent_nft  {
                return get_error("NFT Sent".to_string());
            } else {
                return get_error("NFT not sent".to_string());
            }
        }
        Err(e) => get_error(e),
    }
}
