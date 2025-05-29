use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::from_document;
use serde::Deserialize;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

// In-memory cache for endpoint results
static GET_QUEST_CACHE: Lazy<DashMap<u32, (Instant, Option<QuestDocument>)>> = Lazy::new(DashMap::new);
const GET_QUEST_CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

// IMPORTANT: Ensure the following indexes exist in MongoDB for optimal performance:
// db.quests.createIndex({ id: 1, disabled: 1 })
// These indexes will significantly speed up the aggregation pipeline.

#[route(get, "/get_quest")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    // Check cache first
    if let Some((cached_at, cached_result)) = GET_QUEST_CACHE.get(&query.id).map(|v| v.value().clone()) {
        if cached_at.elapsed() < GET_QUEST_CACHE_TTL {
            if let Some(quest) = cached_result {
                return (StatusCode::OK, Json(quest)).into_response();
            } else {
                return get_error("Quest not found".to_string());
            }
        }
    }

    let collection = state.db.collection::<QuestDocument>("quests");
    let current_time = chrono::Utc::now().timestamp_millis();

    let pipeline = [
        doc! {
            "$match": {
                "disabled": false,
                "id": query.id,
            }
        },
        doc! {
            "$addFields": {
                "expired": {
                    "$cond": [
                        {
                            "$and": [
                               doc! {
                                    "$gte": [
                                        "$expiry",
                                        0
                                    ]
                                },
                                doc! {
                                    "$lt": [
                                        "$expiry",
                                        current_time
                                    ]
                                }
                            ]
                        },
                        true,
                        false
                    ]
                }
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(mut quest) = from_document::<QuestDocument>(document) {
                            if let Some(expiry) = &quest.expiry {
                                quest.expiry_timestamp = Some(expiry.to_string());
                            }
                            // Store in cache
                            GET_QUEST_CACHE.insert(query.id, (Instant::now(), Some(quest.clone())));
                            return (StatusCode::OK, Json(quest)).into_response();
                        }
                    }
                    _ => continue,
                }
            }
            // Store not found in cache
            GET_QUEST_CACHE.insert(query.id, (Instant::now(), None));
            get_error("Quest not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
// Pipeline optimization note:
// - If the quests collection is very large, consider pre-aggregating stats in a background job.
// - Use $project early in the pipeline to reduce memory usage if possible.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
