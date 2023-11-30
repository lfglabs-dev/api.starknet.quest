use crate::models::{BoostTable, QuestDocument};
use crate::{models::AppState, utils::get_error};
use axum::extract::Query;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::{StreamExt};
use mongodb::bson::doc;
use mongodb::bson::from_document;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetQuestForBoostQuery {
    boost_id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestForBoostQuery>,
) -> impl IntoResponse {
    let boost_id = query.boost_id;
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "id": boost_id
            }
        },
        doc! {
            "$unwind": doc! {
                "path": "$quests"
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quests",
                "let": doc! {
                    "task_id": "$quests"
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "$expr": doc! {
                                "$eq": [
                                    "$id",
                                    "$$task_id"
                                ]
                            }
                        }
                    }
                ],
                "as": "quest"
            }
        },
        doc! {
            "$group": doc! {
                "_id": 0,
                "quest_list": doc! {
                    "$push": doc! {
                        "$arrayElemAt": [
                            "$quest",
                            0
                        ]
                    }
                }
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0
            }
        },
        doc! {
            "$project": doc! {
                "quests": doc! {
                    "$map": doc! {
                        "input": "$quest_list",
                        "as": "item",
                        "in": doc! {
                            "id": "$$item.id",
                            "category": "$$item.category",
                            "name": "$$item.name",
                            "desc": "$$item.desc",
                            "issuer": "$$item.issuer",
                            "logo": "$$item.logo",
                            "rewards_img": "$$item.rewards_img",
                            "rewards_title": "$$item.rewards_title",
                            "img_card": "$$item.img_card",
                            "title_card": "$$item.title_card",
                            "hidden": "$$item.hidden",
                            "disabled": "$$item.disabled",
                            "expiry": "$$item.expiry",
                            "expiry_timestamp": "$$item.expiry_timestamp",
                            "mandatory_domain": "$$item.mandatory_domain",
                            "expired": "$$item.expired",
                            "experience": "$$item.experience",
                            "rewards_title": "$$item.rewards_title",
                            "rewards_nfts": "$$item.rewards_nfts",
                            "rewards_description": "$$item.rewards_description",
                            "rewards_endpoint": "$$item.rewards_endpoint",
                        }
                    }
                }
            }
        },
    ];
    let collection = state.db.collection::<BoostTable>("boosts");
    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut res: Vec<QuestDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                let document = result.unwrap();
                let quest_list = document.get_array("quests").unwrap();
                for quest in quest_list {
                    let quest_doc = quest.as_document().unwrap();
                    let quest = from_document::<QuestDocument>(quest_doc.clone()).unwrap();
                    res.push(quest);
                }
            }
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(_) => get_error("Error querying boosts".to_string()),
    }
}
