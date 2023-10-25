use std::sync::Arc;

use crate::{
    models::{AppState, VerifyAchievementQuery},
    utils::{get_error, to_hex, AchievementsTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{NaiveDateTime, Utc};
use serde_json::json;
use starknet::core::types::FieldElement;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    if !(14..=16).contains(&achievement_id) {
        return get_error("Invalid achievement id".to_string());
    }

    let url = format!(
        "https://api.starkscan.co/api/v0/transactions?from_block=1&limit=1&contract_address={}&order_by=asc",
        to_hex(addr)
    );
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("accept", "application/json")
        .header("x-api-key", state.conf.starkscan.api_key.clone())
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(timestamp) = json["data"][0]["timestamp"].as_i64() {
                    let dt = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
                    let current_time = Utc::now().naive_utc();
                    let duration = current_time - dt;
                    let days_passed = duration.num_days();

                    if (achievement_id == 14 && days_passed >= 90)
                        || (achievement_id == 15 && days_passed >= 180)
                        || (achievement_id == 16 && days_passed >= 365)
                    {
                        match state
                            .upsert_completed_achievement(addr, achievement_id)
                            .await
                        {
                            Ok(_) => {
                                (StatusCode::OK, Json(json!({"achieved": true}))).into_response()
                            }
                            Err(e) => get_error(format!("{}", e)),
                        }
                    } else {
                        get_error("Your wallet is too recent".to_string())
                    }
                } else {
                    get_error("No value found for this address".to_string())
                }
            }
            Err(e) => get_error(format!(
                "Failed to get JSON response from Starkscan api: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch Starkscan api: {}", e)),
    }
}
