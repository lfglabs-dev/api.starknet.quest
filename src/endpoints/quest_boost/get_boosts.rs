use crate::models::BoostTable;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::from_document;
use std::sync::Arc;

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pipeline = vec![doc! {
       "$match": {
                "expiry":{
                    "$lt": Utc::now().timestamp_millis()
                },
                "winner": {
                    "$eq": null,
                },
            }
    }];
    let collection = state.db.collection::<BoostTable>("boosts");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<BoostTable> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        quests.push(from_document(document).unwrap());
                    }
                    _ => continue,
                }
            }
            if quests.len() == 0 {
                return get_error("No boosts found".to_string());
            }
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying boosts".to_string()),
    }
}
