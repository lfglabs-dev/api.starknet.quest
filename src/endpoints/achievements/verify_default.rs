use std::sync::Arc;

use crate::{
    models::{AchievedDocument, AppState, VerifyAchievementQuery},
    utils::{get_error, AchievementsTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::bson::doc;
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
    let achieved_collection = state.db.collection::<AchievedDocument>("achieved");
    let filter = doc! {
        "addr": FieldElement::to_string(&addr),
        "achievement_id": achievement_id
    };
    match achieved_collection.find_one(filter, None).await {
        Ok(Some(_)) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
        Ok(None) => match state.get_achievement(achievement_id).await {
            Ok(Some(achievement)) => {
                // todo: add verifying logic here
                match state
                    .upsert_completed_achievement(addr, achievement_id)
                    .await
                {
                    Ok(_) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            }
            Ok(None) => get_error("Achievement not found".to_string()),
            Err(e) => get_error(format!("Error querying achievement : {}", e)),
        },
        Err(e) => get_error(format!("Error querying user achievement : {}", e)),
    }
}
