use crate::models::QuestDocument;
use crate::{models::AppState, utils::get_error};
use axum::response::{IntoResponse, Json};
use axum::{extract::State, http::StatusCode};
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTItem {
    img: String,
    level: u32,
}

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pipeline = vec![
        doc! {
            "$match": {
                "disabled": false,
                "hidden": false,
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
            let mut res: HashMap<String, Vec<QuestDocument>> = HashMap::new();
            for quest in quests {
                let category = quest.category.clone();
                if res.contains_key(&category) {
                    let quests = res.get_mut(&category).unwrap();
                    quests.push(quest);
                } else {
                    res.insert(category, vec![quest]);
                }
            }
            if res.is_empty() {
                get_error("No quests found".to_string())
            } else {
                (StatusCode::OK, Json(res)).into_response()
            }
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
