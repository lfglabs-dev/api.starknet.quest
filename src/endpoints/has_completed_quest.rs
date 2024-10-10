use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HasCompletedQuestsQuery {
    addr: FieldElement,
    quest_id: u32,
}

#[route(get, "/has_completed_quest")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HasCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let quest_id = query.quest_id;
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "address": address,
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
            "$project": doc! {
                "_id": 0,
                "address": 1,
                "task_id": 1,
                "quest_id": "$associatedTask.quest_id"
            }
        },
        doc! {
            "$group": doc! {
                "_id": "$quest_id",
                "done": doc! {
                    "$sum": 1
                }
            }
        },
        doc! {
            "$match": doc! {
                "_id": quest_id,
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
            "$project": doc! {
                "_id": 0,
                "result": doc! {
                    "$cond": doc! {
                        "if": doc! {
                            "$eq": [
                                doc! {
                                    "$size": "$tasks"
                                },
                                "$done"
                            ]
                        },
                        "then": true,
                        "else": false
                    }
                }
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(cursor) => {
            let mut cursor = cursor;
            let mut result = false;
            while let Some(doc) = cursor.try_next().await.unwrap() {
                result = doc.get("result").unwrap().as_bool().unwrap();
            }
            let response = serde_json::json!({ "completed": result });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(_) => get_error("Error querying status".to_string()),
    }
}
