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
use crate::utils::{CompletedTasksTrait, fetch_json_from_url};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EmailQuery>,
) -> impl IntoResponse {
    let task_id = 87;

    // add check for empty email
    if query.email == "" {
        get_error("Please enter your email".to_string());
    }

    // prepare url to make get request to focus tree api for verification
    let url = format!(
        "{}/{}", state.conf.quests.focustree.api_endpoint,
        query.email
    );

    match fetch_json_from_url(url).await {
        Ok(response) => {
            let error_message = response.get("message").unwrap().as_str().unwrap();

            /*
             focus tree will return a message with 403 response if email address if empty
             Something like the below -
                {
                    "message": "Missing Authentication Token"
                }
              The below code will check if the message ha some value and return an error
            */
            if error_message.len() > 0 {
                return get_error("Couldn't verify if user signed up".to_string());
            }

            // check if user has signed up
            let is_signed_up = response.get("hasSignedUp").unwrap().as_bool().unwrap();
            return if is_signed_up {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => {
                        (StatusCode::OK, Json(json!({"res": true}))).into_response()
                    }
                    Err(e) => {
                        get_error(format!("{}", e))
                    }
                }
            } else {
                get_error("Failed to get user sign up status".to_string())
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
