use std::sync::Arc;

use crate::{
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
use serde_json::json;
use starknet::core::types::FieldElement;

#[route(
    get,
    "/achievements/verify_achieved_quests",
    crate::endpoints::achievements::verify_achieved_quests
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    if !(20..=22).contains(&achievement_id) {
        return get_error("Invalid achievement id".to_string());
    }

    let url = format!(
        "{}/get_completed_quests?addr={}",
        state.conf.variables.api_link, addr
    );
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => match response.json::<Vec<u32>>().await {
            Ok(quests) => {
                if quests.is_empty() {
                    return get_error("You have not completed any quests.".to_string());
                }

                if (achievement_id == 20 && quests.len() >= 5)
                    || (achievement_id == 21 && quests.len() >= 10)
                    || (achievement_id == 22 && quests.len() >= 20)
                {
                    match state
                        .upsert_completed_achievement(addr, achievement_id)
                        .await
                    {
                        Ok(_) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
                        Err(e) => get_error(format!("{}", e)),
                    }
                } else {
                    get_error("You have not completed enough quests.".to_string())
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
