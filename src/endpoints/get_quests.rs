use crate::models::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTItem {
    img: String,
    level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestDocument {
    id: u32,
    name: String,
    desc: String,
    issuer: String,
    category: String,
    rewards_endpoint: String,
    logo: String,
    rewards_img: String,
    rewards_title: String,
    rewards_nfts: Vec<NFTItem>,
}

#[derive(Serialize)]
pub struct QueryError {
    error: String,
}

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    match collection.find(None, None).await {
        Ok(cursor) => {
            let quests: Vec<QuestDocument> = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
            if quests.is_empty() {
                let error = QueryError {
                    error: String::from("No quests found"),
                };
                (StatusCode::OK, Json(error)).into_response()
            } else {
                (StatusCode::OK, Json(quests)).into_response()
            }
        }
        Err(_) => {
            let error = QueryError {
                error: String::from("Error querying quests"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
