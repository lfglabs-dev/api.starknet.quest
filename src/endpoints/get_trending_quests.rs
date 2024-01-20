use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTrendingQuestsQuery {
    addr: Option<FieldElement>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetTrendingQuestsQuery>,
) -> impl IntoResponse {
    // Addr might not exist
    let address = match query.addr {
        Some(addr) => addr.to_string(),
        None => "".to_string(),
    };
    let mut pipeline = vec![
        doc! {
            "$match": {
                "disabled": false,
                "hidden": false,
                "is_trending": true,
            }
        },
        doc! {
            "$addFields": {
                "expired": {
                    "$cond": [
                        {
                            "$and": [
                                { "$gte": ["$expiry", 0] },
                                { "$lt": ["$expiry", "$$NOW"] },
                            ]
                        },
                        true,
                        false
                    ]
                }
            }
        },
        doc! {
            "$match": {
                "expired": false,
            }
        },
    ];

    // If address is provided, filter out quests that the user has already completed
    if !address.is_empty() {
        pipeline.extend_from_slice(&[
            doc! {
                "$lookup": {
                        "from": "tasks",
                        "localField": "id",
                        "foreignField": "quest_id",
                        "as": "tasks"
                }
            },
            doc! {
                "$lookup": doc! {
                "from": "completed_tasks",
                "let": {
                    "task_ids": {
                        "$map": {
                            "input": "$tasks",
                            "as": "taskObj",
                            "in": "$$taskObj.id" // Extract the id from each object in the tasks array
                        }
                    }
                },
                "pipeline" : [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    {
                                    "$in": ["$task_id", "$$task_ids"],
                                    },
                                    {
                                    "$eq": ["$address", address],
                                    }
                                ]
                            }
                        }
                    }
                ],
                "as": "completed_tasks"
                }
            },
            doc! {
                "$match": {
                    "$expr": {
                        "$ne": [
                            {
                                "$size": "$tasks",
                            },
                            {
                                "$size": "$completed_tasks",
                            },
                        ],
                    },
                }
            },
        ]);
    }

    let collection = state.db.collection::<QuestDocument>("quests");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<QuestDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(quest) = from_document::<QuestDocument>(document) {
                            quests.push(quest);
                        }
                    }
                    _ => continue,
                }
            }
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
