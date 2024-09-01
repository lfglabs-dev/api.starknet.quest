use std::sync::Arc;

use crate::{
    common::has_deployed_time::execute_has_deployed_time,
    models::{AppState, VerifyAchievementQuery},
    utils::{get_error, AchievementsTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use chrono::{Utc, DateTime};
use serde_json::json;
use starknet::core::types::FieldElement;

#[route(get, "/achievements/verify_seniority")]
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

    match execute_has_deployed_time(state.clone(), &query.addr).await {
        Ok(timestamp) => {
            
            let date_time = DateTime::from_timestamp(timestamp as i64, 0).unwrap();
            let current_time = Utc::now();
            let duration = current_time - date_time;
            let days_passed = duration.num_days();
            if (achievement_id == 14 && days_passed >= 90)
                || (achievement_id == 15 && days_passed >= 180)
                || (achievement_id == 16 && days_passed >= 365)
            {
                match state
                    .upsert_completed_achievement(addr, achievement_id)
                    .await
                {
                    Ok(_) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Your wallet is too recent".to_string())
            }
        }
        Err(e) => get_error(e),
    }
}