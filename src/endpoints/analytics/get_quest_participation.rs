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

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

#[route(
    get,
    "/analytics/get_quest_participation"
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
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
                        task_activity.push(document);
                    }
                    _ => continue,
                }
            }
            return (StatusCode::OK, Json(task_activity)).into_response();
        }
        Err(_) => get_error("Error querying tasks".to_string()),
    }
}
