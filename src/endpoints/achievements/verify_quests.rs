use std::sync::Arc;

use crate::{
    models::{AppState, VerifyAchievementQuery},
    utils::{get_error},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use serde_json::json;
use starknet::core::types::FieldElement;


fn get_number_of_quests(id: u32) -> u32 {
    return match id {
        1 => 1,
        2 => 3,
        3 => 10,
        4 => 25,
        5 => 50,
        _ => 0,
    };
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    let quests_threshold = get_number_of_quests(achievement_id);

    // check valid achievement id
    // if !(17..=19).contains(&achievement_id) {
    //     return get_error("Invalid achievement id".to_string());
    // }

    let pipeline = vec![
        doc! {
            "$match": doc! {
                "address": addr.to_string()
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
            "$count": "total"
        },
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");

    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut total = 0;
            while let Some(result) = cursor.try_next().await.unwrap() {
                total = result.get("total").unwrap().as_i32().unwrap() as u32;
            }
            if total < quests_threshold {
                return get_error("User hasn't completed all tasks".into());
            }
            (StatusCode::OK, Json(json!({"achieved": true})))
                .into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
