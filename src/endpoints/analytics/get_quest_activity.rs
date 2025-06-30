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
static QUEST_ACTIVITY_CACHE: Lazy<DashMap<u32, (Instant, Vec<serde_json::Value>)>> = Lazy::new(DashMap::new);
const CACHE_TTL: Duration = Duration::from_secs(60); // 1 minute cache

// IMPORTANT: Ensure the following indexes exist in MongoDB for optimal performance:
// db.completed_tasks.createIndex({ task_id: 1 })
// db.completed_tasks.createIndex({ timestamp: 1 })
// db.completed_tasks.createIndex({ address: 1 })
// These indexes will significantly speed up the aggregation pipeline.

#[route(get, "/analytics/get_quest_activity")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let current_time = chrono::Utc::now().timestamp_millis();
    let quest_id = query.id;

    // Check cache first
    if let Some((cached_at, cached_result)) = QUEST_ACTIVITY_CACHE.get(&quest_id).map(|v| v.value().clone()) {
        if cached_at.elapsed() < CACHE_TTL {
            return (StatusCode::OK, Json(cached_result)).into_response();
        }
    }

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
            "$replaceRoot": doc! {
                "newRoot": doc! {
                    "$mergeObjects": [
                        "$$ROOT",
                        "$matching_documents"
                    ]
                }
            }
        },
        doc! {
            "$group": doc! {
                "_id": doc! {
                    "_id": "$address",
                    "ids": "$ids"
                },
                "maxTimestamp": doc! {
                    "$max": "$timestamp"
                },
                "tasks": doc! {
                    "$addToSet": "$task_id"
                },
                "count": doc! {
                    "$sum": 1
                }
            }
        },
        doc! {
            "$addFields": doc! {
                "createdDate": doc! {
                    "$toDate": "$maxTimestamp"
                }
            }
        },
        doc! {
            "$match": doc! {
                "$expr": doc! {
                    "$and": [
                        doc! {
                            "$eq": [
                                doc! {
                                    "$size": "$tasks"
                                },
                                doc! {
                                    "$size": "$_id.ids"
                                }
                            ]
                        }
                    ]
                }
            }
        },
        doc! {
            "$group": doc! {
                "_id": doc! {
                    "$dateToString": doc! {
                        "format": "%Y-%m-%d %d",
                        "date": "$createdDate"
                    }
                },
                "count": doc! {
                    "$sum": 1
                }
            }
        },
        doc! {
            "$sort": doc! {
                "_id": 1
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "date": "$_id",
                "participants": "$count"
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
            let mut day_wise_distribution = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        // Convert Document to serde_json::Value
                        let value: serde_json::Value = match serde_json::to_value(&document) {
                            Ok(val) => val,
                            Err(_) => continue,
                        };
                        day_wise_distribution.push(value);
                    }
                    _ => continue,
                }
            }
            // Store in cache
            QUEST_ACTIVITY_CACHE.insert(quest_id, (Instant::now(), day_wise_distribution.clone()));
            return (StatusCode::OK, Json(day_wise_distribution)).into_response();
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}

// Pipeline optimization note:
// - If the completed_tasks collection is very large, consider pre-aggregating daily stats in a background job.
// - If only a subset of fields is needed, use $project early in the pipeline to reduce memory usage.
// - If the pipeline is still slow, consider splitting into multiple smaller queries or using a reporting database.
