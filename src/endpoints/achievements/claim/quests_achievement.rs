use std::sync::Arc;

use crate::utils::{to_hex, AchievementsTrait};
use crate::{
    models::{AppState, VerifyAchievementQuery},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use serde_json::json;
use starknet::core::types::FieldElement;
fn get_number_of_quests(id: u32) -> u32 {
    return match id {
        23 => 1,
        24 => 3,
        25 => 10,
        26 => 25,
        27 => 50,
        _ => 0,
    };
}

#[route(get, "/achievements/claim/quest_achievement")]
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
    if !(23..=27).contains(&achievement_id) {
        return get_error("Invalid achievement id".to_string());
    }

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
            "$group": doc! {
                "_id": null,
                "count": doc! {
                    "$sum": 1
                }
            }
        },
        doc! {
            "$addFields": doc! {
                "result": doc! {
                    "$cond": doc! {
                        "if": doc! {
                            "$gte": [
                                "$count",
                                quests_threshold
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
        Ok(mut cursor) => {
            let mut res = false;
            while let Some(result) = cursor.try_next().await.unwrap() {
                res = result.get("result").unwrap().as_bool().unwrap();
            }
            if !res {
                return get_error("User hasn't completed required number of quests".into());
            }
            let addr_hex = to_hex(addr);
            match state
                .upsert_claimed_achievement(addr_hex, achievement_id)
                .await
            {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            }
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
