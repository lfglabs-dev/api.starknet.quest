use std::sync::Arc;

use crate::{
    common::verify_has_nft::execute_has_nft,
    config::Config,
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

fn get_args(config: Config, achievement_id: u32) -> Result<(FieldElement, u32), String> {
    match achievement_id {
        // ArgentX Xplorer NFTs
        1 => Ok((config.achievements.argent.contract, 1)),
        2 => Ok((config.achievements.argent.contract, 4)),
        3 => Ok((config.achievements.argent.contract, 8)),
        // Braavos Journey NFTs
        4 => Ok((config.achievements.braavos.contract, 1)),
        5 => Ok((config.achievements.braavos.contract, 3)),
        6 => Ok((config.achievements.braavos.contract, 6)),
        _ => Err("Invalid achievement ID".to_string()),
    }
}

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
        Ok(None) => match get_args(state.conf.clone(), achievement_id) {
            Ok((contract, limit)) => {
                let is_achieved = execute_has_nft(&state.conf, addr, contract, limit).await;
                if is_achieved {
                    match state
                        .upsert_completed_achievement(addr, achievement_id)
                        .await
                    {
                        Ok(_) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
                        Err(e) => get_error(format!("{}", e)),
                    }
                } else {
                    (StatusCode::OK, Json(json!({"achieved": false}))).into_response()
                }
            }
            Err(e) => get_error(e),
        },
        Err(e) => get_error(format!("Error querying user achievement : {}", e)),
    }
}
