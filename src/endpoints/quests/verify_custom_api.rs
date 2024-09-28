use std::{sync::Arc, str::FromStr};
use crate::{
    models::{AppState, QuestTaskDocument},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use regex::Regex;
use reqwest::get;
use starknet::core::types::FieldElement;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VerifyCustomApiQuery {
    pub addr: String,
    pub task_id: u32,
}

#[route(get, "/quests/verify_custom_api")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyCustomApiQuery>,
) -> impl IntoResponse {
    let task_id = query.task_id;

    // Get task in db
    let task_collection = state.db.collection("tasks");
    let task: QuestTaskDocument = task_collection
        .find_one(doc! {"id": task_id}, None)
        .await
        .unwrap()
        .unwrap();

    // Check if the task type is "custom_api"
    if task.task_type != Some("custom_api".to_string()) {
        return get_error("Invalid task type.".to_string());
    }

    // Check if the task has the required fields (api_url and regex)
    let api_url = match &task.api_url {
        Some(url) => url,
        None => return get_error("API URL not found.".to_string()),
    };

    let regex_str = match &task.regex {
        Some(rgx) => rgx,
        None => return get_error("Regex not found.".to_string()),
    };

    // Call the specified API
    let response = get(api_url).await;

    match response {
        Ok(res) => {
            let res_text = res.text().await.unwrap();
            
            // Check response against the regex
            let re = Regex::new(regex_str).unwrap();
            if re.is_match(&res_text) {
                // Mark the task as completed
                match state.upsert_completed_task(FieldElement::from_str(&query.addr).unwrap(), task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Response did not match the required pattern.".to_string())
            }
        }
        Err(e) => get_error(format!("Failed to fetch API: {}", e)),
    }
}
