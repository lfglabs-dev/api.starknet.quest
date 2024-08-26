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
use axum_auto_routes::route;
use serde_json::json;
use starknet::core::types::FieldElement;

#[route(get, "/achievements/verify_tvl")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    if !(11..=13).contains(&achievement_id) {
        return get_error("Invalid achievement id".to_string());
    }

    let url = format!(
        "https://stack.starkendefi.xyz/public/aggregates/{}",
        to_hex(addr)
    );
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(total_tvl_dollars) = json["total_tvl_dollars"].as_f64() {
                    if (achievement_id == 11 && total_tvl_dollars >= 100.0)
                        || (achievement_id == 12 && total_tvl_dollars >= 1000.0)
                        || (achievement_id == 13 && total_tvl_dollars >= 10000.0)
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
                        get_error("Your TVL is too low".to_string())
                    }
                } else {
                    get_error("total_tvl_dollars not found or not a float".to_string())
                }
            }
            Err(e) => get_error(format!(
                "Failed to get JSON response from Starkendefi: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch Starkendefi: {}", e)),
    }
}
