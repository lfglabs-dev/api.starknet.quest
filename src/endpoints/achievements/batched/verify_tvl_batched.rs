use std::sync::Arc;

use crate::{
    common::get_achievement::get_achievement,
    models::{AppState, VerifyAchievementBatchedQuery},
    utils::{get_error, to_hex, AchievementsTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use starknet::core::types::FieldElement;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementBatchedQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let url = format!(
        "https://public.starkendefi.xyz/public/aggregates/{}",
        to_hex(addr)
    );
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(total_tvl_dollars) = json["total_tvl_dollars"].as_f64() {
                    if total_tvl_dollars < 100.0 {
                        return get_error("Your TVL is too low".to_string());
                    }

                    match get_achievement(&state, &query.addr, query.category_id).await {
                        Ok(achievements) => {
                            let mut achieved: Vec<u32> = vec![];
                            for achievement in achievements.achievements {
                                if !achievement.completed
                                    && ((achievement.id == 11 && total_tvl_dollars >= 100.0)
                                        || (achievement.id == 12 && total_tvl_dollars >= 1000.0)
                                        || (achievement.id == 13 && total_tvl_dollars >= 10000.0))
                                {
                                    match state
                                        .upsert_completed_achievement(addr, achievement.id)
                                        .await
                                    {
                                        Ok(_) => {
                                            achieved.push(achievement.id);
                                        }
                                        Err(e) => return get_error(format!("{}", e)),
                                    }
                                }
                            }
                            (StatusCode::OK, Json(json!({ "achieved": achieved }))).into_response()
                        }
                        Err(e) => get_error(e),
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
