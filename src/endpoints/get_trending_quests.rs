use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::from_document;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTItem {
    img: String,
    level: u32,
}

#[route(get, "/get_trending_quests", crate::endpoints::get_trending_quests)]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pipeline = vec![
        doc! {
            "$match": {
                "disabled": false,
                "hidden": false,
                "is_trending": true,
            }
        },
        doc! {
            "$addFields": {
                "expired": {
                    "$cond": [
                        {
                            "$and": [
                                { "$gte": ["$expiry", 0] },
                                { "$lt": ["$expiry", "$$NOW"] },
                            ]
                        },
                        true,
                        false
                    ]
                }
            }
        },
        doc! {
            "$match": {
                "expired": false,
            }
        },
    ];

    let collection = state.db.collection::<QuestDocument>("quests");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<QuestDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(quest) = from_document::<QuestDocument>(document) {
                            quests.push(quest);
                        }
                    }
                    _ => continue,
                }
            }
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
