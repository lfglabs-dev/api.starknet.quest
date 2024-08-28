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

#[route(get, "/achievements/verify_quests")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    let hex_addr = to_hex(addr);
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
                "achieved": doc! {
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
                },
                "id": achievement_id
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "achieved": "$achieved",
                "id": "$id"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "achievements",
                "localField": "id",
                "foreignField": "id",
                "as": "achievement"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "claimed_achievements",
                "let": doc! {
                    "local_id": "$id",
                    "local_address": &hex_addr
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "$expr": doc! {
                                "$and": [
                                    doc! {
                                        "$eq": [
                                            "$id",
                                            "$$local_id"
                                        ]
                                    },
                                    doc! {
                                        "$eq": [
                                            "$address",
                                            "$$local_address"
                                        ]
                                    },
                                ]
                            }
                        }
                    }
                ],
                "as": "claimed_achievement"
            }
        },
        doc! {
            "$project": doc! {
                "achieved": "$achieved",
                "claimed": doc! {
                    "$cond": doc! {
                        "if": doc! {
                            "$or": [
                                doc! {
                                    "$gte": [
                                        doc! {
                                            "$size": "$claimed_achievement"
                                        },
                                        1
                                    ]
                                },
                                doc! {
                                    "$eq": [
                                        "$achieved",
                                        false
                                    ]
                                }
                            ]
                        },
                        "then": false,
                        "else": doc! {
                            "$arrayElemAt": [
                                "$achievement.claimable",
                                0
                            ]
                        }
                    }
                }
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");

    return match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                let document = result;
                let achieved = document.get("achieved").unwrap().clone();
                let claimed = document.get("claimed").unwrap().clone();
                let response = json!({
                    "achieved": achieved,
                    "claimed": claimed
                });
                if !achieved.as_bool().unwrap() {
                    return (StatusCode::OK, Json(response)).into_response();
                }

                return match state
                    .upsert_completed_achievement(addr, achievement_id)
                    .await
                {
                    Ok(_) => (StatusCode::OK, Json(response)).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                };
            }
            get_error("Error querying quests".to_string())
        }
        Err(_) => get_error("Error querying quests".to_string()),
    };
}
