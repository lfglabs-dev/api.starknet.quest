use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]

pub struct GetCompletedQuestsQuery {
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let pipeline = vec![
        // Existing pipeline to get completed quests
        doc! {
            "$match": doc! {
                "address": address
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
        // New stages to match completed quests with boosts
        doc! {
            "$lookup": doc! {
                "from": "boosts",
                "let": { "completedQuest": "$quest_id" },
                "pipeline": vec![
                    doc! {
                        "$match": doc! {
                            "$expr": doc! {
                                "$in": [ "$$completedQuest", "$quests" ]
                            }
                        }
                    },
                ],
                "as": "matchedBoosts"
            }
        },
        doc! {
            "$unwind": "$matchedBoosts"
        },
        doc! {
            "$group": doc! {
                "_id": "$matchedBoosts.id",
                "boostDetails": doc! { "$first": "$matchedBoosts" },
                "matchedQuestsCount": doc! { "$sum": 1 }
            }
        },
        doc! {
            "$match": doc! {
                "$expr": doc! {
                    "$eq": [ "$matchedQuestsCount", doc! { "$size": "$boostDetails.quests" } ]
                }
            }
        },
        doc! {
            "$project": doc! {
                "boost_id": "$_id",
                "_id": 0
            }
        },
    ];
    let collection = state.db.collection::<Document>("completed_tasks");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut boosts: Vec<u32> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                boosts.push(result.get("boost_id").unwrap().as_i64().unwrap() as u32);
            }
            (StatusCode::OK, Json(boosts)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
