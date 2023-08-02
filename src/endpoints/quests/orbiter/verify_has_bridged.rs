use std::sync::Arc;

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

#[derive(Debug, serde::Deserialize)]
struct UserTaskStatus {
    address: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    userTaskStatus: UserTaskStatus,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    data: Data,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 32;
    let hex_addr = to_hex(query.addr);

    let graphql_url = "https://actapi.orbiter.finance/graphql/activity";
    let graphql_query = r#"
        query info($address: String!) {
            userTaskStatus(address: $address, taskId: "5", verify: "aIHcjkNqpcD") {
                address
            }
        }
    "#;
    let variables = serde_json::json!({ "address": hex_addr });

    let client = reqwest::Client::new();
    let response_result = client
        .post(graphql_url)
        .json(&serde_json::json!({
            "query": graphql_query,
            "variables": variables
        }))
        .send()
        .await;

    match response_result {
        Ok(response) => match response.json::<Response>().await {
            Ok(res) => {
                if res.data.userTaskStatus.address.is_empty() {
                    get_error("You haven't bridge ETH to Starknet using Orbiter.".to_string())
                } else {
                    match state.upsert_completed_task(query.addr, task_id).await {
                        Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                        Err(e) => get_error(format!("{}", e)),
                    }
                }
            }
            Err(e) => get_error(format!(
                "Failed to get JSON response while fetching user info: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch user info: {}", e)),
    }
}
