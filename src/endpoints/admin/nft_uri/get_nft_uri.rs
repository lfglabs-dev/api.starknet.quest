use crate::models::NFTUri;
use crate::{models::AppState, utils::get_error};
use axum::Json;
use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::IntoResponse,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetNFTUriParams {
    quest_id: i64,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetNFTUriParams>,
) -> impl IntoResponse {
    let collection = state.db.collection::<NFTUri>("nft_uri");
    
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

