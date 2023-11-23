use std::sync::Arc;

use crate::{
    models::{AchievedDocument, AppState, VerifyAchievementQuery},
    utils::{get_error, to_hex, AchievementsTrait},
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
use crate::utils::fetch_json_from_url;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    let duck_id = 10;
    let briq_nft_id = 8;
    if achievement_id != duck_id && achievement_id != briq_nft_id {
        return get_error("Invalid achievement id".to_string());
    }

    let achieved_collection = state.db.collection::<AchievedDocument>("achieved");
    let filter = doc! {
        "addr": FieldElement::to_string(&addr),
        "achievement_id": achievement_id
    };
    match achieved_collection.find_one(filter, None).await {
        Ok(Some(_)) => (StatusCode::OK, Json(json!({"achieved": true}))).into_response(),
        Ok(None) => {
            let url = format!(
                "https://api.briq.construction/v1/user/data/starknet-mainnet/{}",
                to_hex(addr)
            );
            match fetch_json_from_url(url).await {
                Ok(response) => {
                    if let Some(sets) = response.get("sets") {
                        match sets {
                            serde_json::Value::Array(sets_array) => {
                                for set in sets_array.iter() {
                                    if let serde_json::Value::String(set_str) = set {
                                        let url = format!(
                                            "https://api.briq.construction/v1/metadata/starknet-mainnet/{}",
                                            set_str
                                        );
                                        match fetch_json_from_url(url).await {
                                            Ok(metadata_response) => {
                                                if let Some(properties) =
                                                    metadata_response.get("properties")
                                                {
                                                    let is_duck =
                                                        check_for_ducks(&properties).await;
                                                    if (achievement_id == duck_id && is_duck)
                                                        || (achievement_id == briq_nft_id
                                                            && !is_duck)
                                                    {
                                                        match state
                                                            .upsert_completed_achievement(
                                                                addr,
                                                                achievement_id,
                                                            )
                                                            .await
                                                        {
                                                            Ok(_) => {
                                                                return (
                                                                    StatusCode::OK,
                                                                    Json(json!({"achieved": true})),
                                                                )
                                                                    .into_response();
                                                            }
                                                            Err(e) => {
                                                                return get_error(format!("{}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => return get_error(e),
                                        }
                                    }
                                }
                            }
                            _ => {
                                return get_error("No Briq sets founds".to_string());
                            }
                        }
                    }
                }
                Err(e) => return get_error(e),
            }
            (StatusCode::OK, Json(json!({"achieved": false}))).into_response()
        }
        Err(e) => get_error(format!("Error querying user briq NFTs : {}", e)),
    }
}

pub async fn check_for_ducks(properties: &serde_json::Value) -> bool {
    if let Some(serde_json::Value::Array(value_arr)) =
        properties.get("collections").and_then(|c| c.get("value"))
    {
        return value_arr.iter().any(|val| {
            if let serde_json::Value::String(s) = val {
                s == "Ducks Everywhere"
            } else {
                false
            }
        });
    }
    false
}
