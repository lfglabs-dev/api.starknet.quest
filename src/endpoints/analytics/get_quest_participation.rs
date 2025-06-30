use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
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
static QUEST_PARTICIPATION_CACHE: Lazy<DashMap<u32, (Instant, Vec<serde_json::Value>)>> = Lazy::new(DashMap::new);
const QUEST_PARTICIPATION_CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

// IMPORTANT: Ensure the following indexes exist in MongoDB for optimal performance:
// db.tasks.createIndex({ quest_id: 1 })
// db.completed_tasks.createIndex({ task_id: 1, timestamp: 1 })
// db.quests.createIndex({ id: 1 })
// These indexes will significantly speed up the aggregation pipeline.

#[route(get, "/analytics/get_quest_participation")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    // Check cache first
    if let Some((cached_at, cached_result)) = QUEST_PARTICIPATION_CACHE.get(&query.id).map(|v| v.value().clone()) {
        if cached_at.elapsed() < QUEST_PARTICIPATION_CACHE_TTL {
            return (StatusCode::OK, Json(cached_result)).into_response();
        }
    }
    let current_time = chrono::Utc::now().timestamp_millis();
    let quest_id = query.id;
    let day_wise_distribution = vec![
        doc! {
            "$match": doc! {
                "quest_id": quest_id
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quests",
                "localField": "quest_id",
                "foreignField": "id",
                "as": "questDetails"
            }
        },
        doc! {
            "$set": doc! {
                "expiry": doc! {
                    "$arrayElemAt": [
                        "$questDetails.expiry",
                        0
                    ]
                }
            }
        },
        doc! {
            "$group": doc! {
                "_id": doc! {
                    "expiry": "$expiry"
                },
                "ids": doc! {
                    "$push": "$id"
                },
                "otherDetails": doc! {
                    "$push": "$$ROOT"
                }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "completed_tasks",
                "let": doc! {
                    "localIds": "$ids",
                    "expiry": "$_id.expiry"
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "$expr": doc! {
                                "$and": [
                                    doc! {
                                        "$in": [
                                            "$task_id",
                                            "$$localIds"
                                        ]
                                    },
                                   doc! {
                                    "$lte": [
                                        "$timestamp",
                                        doc! {
                                            "$ifNull": [
                                                "$$expiry",
                                                current_time
                                            ]
                                        }
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
        doc! {
            "$unwind": "$matching_documents"
        },
        doc! {
            "$group": doc! {
                "_id": "$matching_documents.task_id",
                "count": doc! {
                    "$sum": 1
                },
                "details": doc! {
                    "$first": "$otherDetails"
                }
            }
        },
        doc! {
            "$project": doc! {
                "_id": 1,
                "count": 1,
                "otherDetails": doc! {
                    "$filter": doc! {
                        "input": "$details",
                        "as": "detail",
                        "cond": doc! {
                            "$eq": [
                                "$$detail.id",
                                "$_id"
                            ]
                        }
                    }
                }
            }
        },
        doc! {
            "$unwind": "$otherDetails"
        },
        doc! {
            "$replaceRoot": doc! {
                "newRoot": doc! {
                    "$mergeObjects": [
                        "$matching_documents",
                        "$otherDetails",
                        doc! {
                            "count": "$count"
                        }
                    ]
                }
            }
        },
        doc! {
          "$project": doc! {
              "otherDetails": 0,
              "_id":0,
              "verify_endpoint": 0,
              "verify_endpoint_type": 0,
              "verify_redirect":0,
              "href": 0,
              "cta": 0,
              "id": 0,
              "quest_id": 0,
              "questDetails": 0,
              "expiry":0
            }
        },
    ];

    match state
        .db
        .collection::<QuestTaskDocument>("tasks")
        .aggregate(day_wise_distribution, None)
        .await
    {
        Ok(mut cursor) => {
            let mut task_activity = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        // Convert Document to serde_json::Value
                        let value: serde_json::Value = match serde_json::to_value(&document) {
                            Ok(val) => val,
                            Err(_) => continue,
                        };
                        task_activity.push(value);
                    }
                    _ => continue,
                }
            }
            // Store in cache
            QUEST_PARTICIPATION_CACHE.insert(query.id, (Instant::now(), task_activity.clone()));
            return (StatusCode::OK, Json(task_activity)).into_response();
        }
        Err(_) => get_error("Error querying tasks".to_string()),
    }
}
// Pipeline optimization note:
// - If the completed_tasks or tasks collections are very large, consider pre-aggregating stats in a background job.
// - Use $project early in the pipeline to reduce memory usage if possible.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
