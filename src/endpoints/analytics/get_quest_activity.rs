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
"/analytics/get_quest_activity",
crate::endpoints::analytics::get_quest_activity
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
                        day_wise_distribution.push(document);
                    }
                    _ => continue,
                }
            }
            return (StatusCode::OK, Json(day_wise_distribution)).into_response();
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
