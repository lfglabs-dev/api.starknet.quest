use crate::models::{NFTUri, QuestDocument};
use crate::utils::verify_quest_auth;
use crate::models::JWTClaims;
use crate::{models::AppState, utils::get_error};
use axum::Json;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Router,
    routing::get,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

#[derive(Deserialize)]
pub struct GetNFTUriParams {
    quest_id: i64,
}

async fn get_nft_uri_handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<GetNFTUriParams>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<NFTUri>("nft_uri");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &params.quest_id).await;
    if !res {
        return get_error("Error retrieving task".to_string());
    };

    let filter = doc! { "quest_id": params.quest_id };
    match collection.find_one(filter, None).await {
        Ok(Some(document)) => (
            StatusCode::OK,
            Json(json!({"nft_uri": document})).into_response(),
        )
            .into_response(),
        Ok(None) => get_error("NFT URI not found".to_string()),
        Err(_) => get_error("Error retrieving NFT URI".to_string()),
    }
}

pub fn get_nft_uri_router() -> Router {
    Router::new().route("/nft_uri", get(get_nft_uri_handler))
}
