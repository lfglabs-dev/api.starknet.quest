use std::sync::Arc;

use crate::{
    endpoints::quests::starkfighter::models::ScoreResponse,
    models::{AppState, CompletedTaskDocument, VerifyQuery},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, options::UpdateOptions};
use reqwest::Client as HttpClient;
use serde_json::json;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 3;
    let addr = &query.addr;

    let client = HttpClient::new();
    let body = json!({
        "user_addr": addr,
    });

    let response = client
        .post(format!(
            "{}fetch_user_score",
            state.conf.quests.starkfighter_server
        ))
        .header(CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ScoreResponse>().await {
                    Ok(player_score) => {
                        if player_score.score > 50.into() {
                            let completed_tasks_collection = state
                                .db
                                .collection::<CompletedTaskDocument>("completed_tasks");
                            let filter = doc! { "address": addr.to_string(), "task_id": task_id };
                            let update = doc! { "$setOnInsert": { "address": addr.to_string(), "task_id": task_id } };
                            let options = UpdateOptions::builder().upsert(true).build();

                            let result = completed_tasks_collection
                                .update_one(filter, update, options)
                                .await;

                            match result {
                                Ok(_) => {
                                    (StatusCode::OK, Json(json!({"res": true}))).into_response()
                                }
                                Err(e) => get_error(format!("{}", e)),
                            }
                        } else {
                            get_error("You have a lower score".to_string())
                        }
                    }
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("You have not played".to_string())
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
