use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use dashmap::DashMap;
use futures::StreamExt;
use mongodb::bson::doc;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

// In-memory cache for endpoint results
static QUEST_PARTICIPATION_CACHE: Lazy<DashMap<u32, (Instant, Vec<serde_json::Value>)>> =
    Lazy::new(DashMap::new);
const QUEST_PARTICIPATION_CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

#[route(get, "/analytics/get_quest_participation")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    // Check cache first
    if let Some((cached_at, cached_result)) = QUEST_PARTICIPATION_CACHE
        .get(&query.id)
        .map(|v| v.value().clone())
    {
        if cached_at.elapsed() < QUEST_PARTICIPATION_CACHE_TTL {
            return (StatusCode::OK, Json(cached_result)).into_response();
        }
    }

    let current_time = chrono::Utc::now().timestamp_millis();

    let quest_id = query.id;
    let pipeline = vec![
        doc! { "$match": { "quest_id": quest_id } },
        doc! {
            "$lookup": {
                "from": "quests",
                "localField": "quest_id",
                "foreignField": "id",
                "as": "questDetails"
            }
        },
        doc! {
            "$set": {
                "expiry": { "$arrayElemAt": ["$questDetails.expiry", 0] }
            }
        },
        doc! {
            "$group": {
                "_id": { "expiry": "$expiry" },
                "ids": { "$push": "$id" },
                "otherDetails": { "$push": "$$ROOT" }
            }
        },
        doc! {
            "$lookup": {
                "from": "completed_tasks",
                "let": {
                    "localIds": "$ids",
                    "expiry": "$_id.expiry"
                },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$in": ["$task_id", "$$localIds"] },
                                    {
                                        "$lte": [
                                            "$timestamp",
                                            { "$ifNull": ["$$expiry", current_time] }
                                        ]
                                    }
                                ]
                            }
                        }
                    }
                ],
                "as": "matching_documents"
            }
        },
        doc! { "$unwind": "$matching_documents" },
        doc! {
            "$group": {
                "_id": "$matching_documents.task_id",
                "count": { "$sum": 1 },
                "details": { "$first": "$otherDetails" }
            }
        },
        doc! {
            "$project": {
                "_id": 1,
                "count": 1,
                "otherDetails": {
                    "$filter": {
                        "input": "$details",
                        "as": "detail",
                        "cond": { "$eq": [ "$$detail.id", "$_id" ] }
                    }
                }
            }
        },
        doc! { "$unwind": "$otherDetails" },
        doc! {
            "$replaceRoot": {
                "newRoot": {
                    "$mergeObjects": [
                        "$matching_documents",
                        "$otherDetails",
                        { "count": "$count" }
                    ]
                }
            }
        },
        doc! {
            "$project": {
                "otherDetails": 0,
                "_id": 0,
                "verify_endpoint": 0,
                "verify_endpoint_type": 0,
                "verify_redirect": 0,
                "href": 0,
                "cta": 0,
                "id": 0,
                "quest_id": 0,
                "questDetails": 0,
                "expiry": 0
            }
        },
    ];

    match state
        .db
        .collection::<QuestTaskDocument>("tasks")
        .aggregate(pipeline, None)
        .await
    {
        Ok(mut cursor) => {
            let mut task_activity = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => match serde_json::to_value(&document) {
                        Ok(json) => task_activity.push(json),
                        Err(e) => {
                            state.logger.warning(format!(
                                "[WARN] Quest ID {} - Skipping doc due to serialization error: {:?}",
                                query.id, e
                            ));
                        }
                    },
                    Err(e) => {
                        state.logger.warning(format!(
                            "[WARN] Quest ID {} - Cursor read error: {:?}",
                            query.id, e
                        ));
                    }
                }
            }

            QUEST_PARTICIPATION_CACHE.insert(query.id, (Instant::now(), task_activity.clone()));

            (StatusCode::OK, Json(task_activity)).into_response()
        }
        Err(_) => get_error("Error querying tasks".to_string()),
    }
}
// Pipeline optimization note:
// - If the completed_tasks or tasks collections are very large, consider pre-aggregating stats in a background job.
// - Use $project early in the pipeline to reduce memory usage if possible.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
