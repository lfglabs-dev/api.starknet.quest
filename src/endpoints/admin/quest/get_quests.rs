use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut pipeline = vec![];
    pipeline.push(doc! {
        "$match": doc! {
            "issuer": "super_user"
        }
    });
    let collection = state.db.collection::<QuestDocument>("quests");

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<QuestDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(mut quest) = from_document::<QuestDocument>(document) {
                            if let Some(expiry) = &quest.expiry {
                                quest.expiry_timestamp = Some(expiry.to_string());
                            }
                            quests.push(quest);
                        }
                    }
                    _ => continue,
                }
            }

            if quests.is_empty() {
                get_error("No quests found".to_string())
            } else {
                (StatusCode::OK, Json(quests)).into_response()
            }
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}

