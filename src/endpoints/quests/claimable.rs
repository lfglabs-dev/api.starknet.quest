use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use crate::models::{Reward, RewardResponse};
use crate::utils::get_nft;
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use starknet::signers::{LocalWallet, SigningKey};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HasCompletedQuestsQuery {
    addr: FieldElement,
    quest_id: u32,
}

#[route(get, "/quests/claimable")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HasCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let quest_id = query.quest_id;
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
                "_id": quest_id
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
                "_id": 1,
                "tasks": "$tasks",
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
        doc! {
            "$unwind": doc! {
                "path": "$tasks"
            }
        },
        doc! {
            "$sort": doc! {
                "tasks.id": -1
            }
        },
        doc! {
            "$limit": 1
        },
        doc! {
            "$lookup": doc! {
                "from": "nft_uri",
                "localField": "_id",
                "foreignField": "quest_id",
                "as": "nft_uri"
            }
        },
        doc! {
            "$project": doc! {
                "result": 1,
                "_id": 0,
                "last_task": "$tasks.id",
                "nft_level": "$nft_uri.id"
            }
        },
        doc! {
            "$unwind": doc! {
                "path": "$nft_level"
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("completed_tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(cursor) => {
            let mut cursor = cursor;
            let mut result = false;
            let mut nft_level = 0;
            let mut last_task = 0;
            while let Some(doc) = cursor.try_next().await.unwrap() {
                result = doc.get("result").unwrap().as_bool().unwrap();
                nft_level = doc.get("nft_level").unwrap().as_i64().unwrap();
                last_task = doc.get("last_task").unwrap().as_i32().unwrap();
            }

            if !result {
                return get_error("User hasn't completed all tasks".to_string());
            }

            let signer = LocalWallet::from(SigningKey::from_secret_scalar(
                state.conf.nft_contract.private_key,
            ));

            let mut rewards = vec![];

            let Ok((token_id, sig)) = get_nft(
                quest_id,
                last_task as u32,
                &query.addr,
                nft_level as u32,
                &signer,
            )
            .await
            else {
                return get_error("Signature failed".into());
            };

            rewards.push(Reward {
                task_id: last_task as u32,
                nft_contract: state.conf.nft_contract.address.clone(),
                token_id: token_id.to_string(),
                sig: (sig.r, sig.s),
            });

            if rewards.is_empty() {
                get_error("No rewards found for this user".into())
            } else {
                (StatusCode::OK, Json(RewardResponse { rewards })).into_response()
            }
        }
        Err(_) => get_error("Error querying status".to_string()),
    }
}
