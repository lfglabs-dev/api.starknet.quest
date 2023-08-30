use std::sync::Arc;

use crate::{
    common::verify_has_nft::execute_has_nft,
    config::Config,
    endpoints::achievements::verify_whitelisted::is_braavos_whitelisted,
    models::{AchievedDocument, AppState, Nft, VerifyAchievementQuery},
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

use super::verify_whitelisted::is_argent_whitelisted;

type NftCheck = fn(&Nft) -> bool;

fn get_args(config: Config, achievement_id: u32) -> Result<(FieldElement, u32, NftCheck), String> {
    let argent_contract = config.achievements.argent.contract;
    let braavos_contract = config.achievements.braavos.contract;

    match achievement_id {
        // ArgentX Xplorer NFTs
        1 => Ok((argent_contract, 1, is_argent_whitelisted)),
        2 => Ok((argent_contract, 4, is_argent_whitelisted)),
        3 => Ok((argent_contract, 8, is_argent_whitelisted)),
        // Braavos Journey NFTs
        4 => Ok((braavos_contract, 1, is_braavos_whitelisted)),
        5 => Ok((braavos_contract, 3, is_braavos_whitelisted)),
        6 => Ok((braavos_contract, 5, is_braavos_whitelisted)),
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
            Ok((contract, limit, is_whitelisted)) => {
                match execute_has_nft(&state.conf, addr, contract, limit, is_whitelisted).await {
                    Ok(is_achieved) => {
                        if is_achieved {
                            match state
                                .upsert_completed_achievement(addr, achievement_id)
                                .await
                            {
                                Ok(_) => (StatusCode::OK, Json(json!({"achieved": true})))
                                    .into_response(),
                                Err(e) => get_error(format!("{}", e)),
                            }
                        } else {
                            (StatusCode::OK, Json(json!({"achieved": false}))).into_response()
                        }
                    }
                    Err(e) => get_error(e),
                }
            }
            Err(e) => get_error(e),
        },
        Err(e) => get_error(format!("Error querying user achievement : {}", e)),
    }
}
