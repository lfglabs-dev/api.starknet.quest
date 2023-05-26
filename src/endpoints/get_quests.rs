use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
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

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    match collection.find(None, None).await {
        Ok(cursor) => {
            let quests: Vec<QuestDocument> = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
            if quests.is_empty() {
                get_error("No quests found".to_string())
            } else {
                (StatusCode::OK, Json(quests)).into_response()
            }
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
