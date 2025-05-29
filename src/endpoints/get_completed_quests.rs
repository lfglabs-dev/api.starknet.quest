use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use axum::http::StatusCode;
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]

pub struct GetCompletedQuestsQuery {
    addr: FieldElement,
}

// In-memory cache for endpoint results
static GET_COMPLETED_QUESTS_CACHE: Lazy<DashMap<String, (Instant, Vec<u32>)>> = Lazy::new(DashMap::new);
const GET_COMPLETED_QUESTS_CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

// IMPORTANT: Ensure the following indexes exist in MongoDB for optimal performance:
// db.completed_tasks.createIndex({ address: 1, task_id: 1 })
// db.tasks.createIndex({ id: 1, quest_id: 1 })
// These indexes will significantly speed up the aggregation pipeline.

#[route(get, "/get_completed_quests")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    // Check cache first
    if let Some((cached_at, cached_result)) = GET_COMPLETED_QUESTS_CACHE.get(&address).map(|v| v.value().clone()) {
        if cached_at.elapsed() < GET_COMPLETED_QUESTS_CACHE_TTL {
            return (StatusCode::OK, Json(cached_result)).into_response();
        }
    }
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "address": &address
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "tasks",
                "localField": "task_id",
                "foreignField": "id",
                "as": "associatedTask"
            }
        },
        doc! {
            "$unwind": "$associatedTask"
        },
        doc! {
            "$group": doc! {
                "_id": "$associatedTask.quest_id",
                "done": doc! {
                    "$sum": 1
                }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "tasks",
                "localField": "_id",
                "foreignField": "quest_id",
                "as": "tasks"
            }
        },
        doc! {
            "$match": doc! {
                "$expr": doc! {
                    "$eq": [
                        "$done",
                        doc! {
                            "$size": "$tasks"
                        }
                    ]
                }
            }
        },
        doc! {
            "$project": doc! {
                "quest_id": "$_id",
                "_id": 0
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<u32> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                quests.push(result.get("quest_id").unwrap().as_i64().unwrap() as u32);
            }
            // Store in cache
            GET_COMPLETED_QUESTS_CACHE.insert(address.clone(), (Instant::now(), quests.clone()));
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
// Pipeline optimization note:
// - If the completed_tasks or tasks collections are very large, consider pre-aggregating stats in a background job.
// - Use $project early in the pipeline to reduce memory usage if possible.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
