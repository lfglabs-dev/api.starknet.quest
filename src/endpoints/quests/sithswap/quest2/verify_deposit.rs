use std::sync::Arc;

use crate::utils::to_hex;
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

fn string_to_float(s: &str) -> Result<f64, std::num::ParseFloatError> {
    s.parse::<f64>()
}

#[route(get, "/quests/sithswap2/verify_deposit")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 151;
    let addr = &query.addr;
    let mut hex_addr = to_hex(*addr);
    // Define the GraphQL endpoint
    let endpoint = &state.conf.quests.sithswap_2.api_endpoint;

    // remove "0x"
    hex_addr.remove(0);
    hex_addr.remove(0);

    // remove leading zeroes
    while hex_addr.starts_with("0") {
        hex_addr.remove(0);
    }

    // add "0x" back
    hex_addr.insert(0, 'x');
    hex_addr.insert(0, '0');

    // Define the GraphQL query
    let graphql_query = format!(
        r#"
        {{
          mints(where: {{to: "{}"}}) {{
            amountUSD
            timestamp
            to
            pair {{
              id
            }}
          }}
        }}
        "#,
        &hex_addr
    );

    // Send the GraphQL query
    let response = reqwest::Client::new()
        .post(endpoint)
        .json(&json!({ "query": graphql_query }))
        .send()
        .await;

    // Check if the response is successful
    let response = match response {
        Ok(response) => response,
        Err(_) => return get_error(format!("Try again later")),
    };

    // Parse the response
    let response = response.json::<serde_json::Value>().await;
    let response = match response {
        Ok(response) => response,
        Err(_) => return get_error(format!("Try again later")),
    };

    let mut total_amount_usd = 0.0;

    // Check if the response contains the data field
    if let Some(data) = response.get("data") {
        // Check if the data field contains the mints field
        if let Some(mints) = data.get("mints") {
            // Iterate over the mints
            for mint in mints.as_array().unwrap() {
                // Check if the mint contains the amountUSD field
                if let Some(amount_usd) = mint.get("amountUSD").and_then(|v| v.as_str()) {
                    match string_to_float(amount_usd) {
                        Ok(amount_usd) => total_amount_usd += amount_usd,
                        Err(_e) => return get_error(format!("Failed to get balance")),
                    }
                }
            }
        }
    }

    // Check if the total_amount_usd is greater than 10
    if total_amount_usd >= 10.0 {
        // Update the completed task
        match state.upsert_completed_task(*&query.addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(_e) => get_error(format!("Failed to get balance")),
        }
    } else {
        get_error("User has not completed the task".to_string())
    }
}
