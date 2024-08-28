use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use axum_auto_routes::route;
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

#[route(get, "/boost/get_completed_boosts")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let pipeline = vec![
        // Existing pipeline to get completed quests
        doc! {
            "$lookup": doc! {
                "from": "tasks",
                "localField": "quests",
                "foreignField": "quest_id", // Replace 'questId' with the actual field name in your tasks collection
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
            "$match": doc! {
                "$expr": {
                    "$eq": [
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
        doc! {
            "$project": {
                "id": 1,
            }
        },
    ];
    let collection = state.db.collection::<Document>("boosts");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut boosts: Vec<u32> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                boosts.push(result.get("id").unwrap().as_i32().unwrap() as u32);
            }
            (StatusCode::OK, Json(boosts)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
