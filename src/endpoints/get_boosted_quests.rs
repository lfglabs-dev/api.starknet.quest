use crate::{models::AppState, utils::get_error};
use axum::{extract::State, response::IntoResponse, Json};

use axum::http::StatusCode;
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

// In-memory cache for endpoint results
static GET_BOOSTED_QUESTS_CACHE: Lazy<DashMap<(), (Instant, Vec<u32>)>> = Lazy::new(DashMap::new);
const GET_BOOSTED_QUESTS_CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

// IMPORTANT: Ensure the following indexes exist in MongoDB for optimal performance:
// db.boosts.createIndex({ quests: 1 })
// These indexes will significantly speed up the aggregation pipeline.

#[route(get, "/get_boosted_quests")]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check cache first (no params, so use unit key)
    if let Some((cached_at, cached_result)) = GET_BOOSTED_QUESTS_CACHE.get(&()).map(|v| v.value().clone()) {
        if cached_at.elapsed() < GET_BOOSTED_QUESTS_CACHE_TTL {
            return (StatusCode::OK, Json(cached_result)).into_response();
        }
    }

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
            // Store in cache
            GET_BOOSTED_QUESTS_CACHE.insert((), (Instant::now(), quests.clone()));
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying boosts".to_string()),
    }
}

// Pipeline optimization note:
// - If the boosts collection is very large, consider pre-aggregating stats in a background job.
// - Use $project early in the pipeline to reduce memory usage if possible.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
