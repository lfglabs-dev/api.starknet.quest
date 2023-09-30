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
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<Document> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                quests.push(result);
            }
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
