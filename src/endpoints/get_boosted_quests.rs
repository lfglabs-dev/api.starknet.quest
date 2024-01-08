use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{State},
    response::IntoResponse,
    Json,
};

use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use reqwest::StatusCode;
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pipeline = vec![
        doc! {
            "$unwind": doc! {
                "path": "$quests"
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "id": "$quests"
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("boosts");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<u32> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                quests.push(result.get("id").unwrap().as_i32().unwrap() as u32);
            }
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying boosts".to_string()),
    }
}
