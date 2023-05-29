use crate::models::RewardResponse;
use crate::utils::get_nft;
use crate::{
    models::{AppState, CompletedTaskDocument, Reward},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::FieldElement,
    signers::{LocalWallet, SigningKey},
};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    id: u32,
    quest_id: u32,
    name: String,
    desc: String,
    href: String,
    cta: Option<String>,
    verify_endpoint: Option<String>,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestProps {
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterRequestProps {
    id: String,
}

#[derive(Deserialize)]
pub struct ClaimableQuery {
    addr: FieldElement,
}

const QUEST_ID: u32 = 123;
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClaimableQuery>,
) -> impl IntoResponse {
    let collection = state
        .db
        .collection::<CompletedTaskDocument>("completed_tasks");
    let pipeline = vec![
        doc! {
            "$match": {
                "address": &query.addr.to_string(),
            },
        },
        doc! {
            "$lookup": {
                "from": "tasks",
                "localField": "task_id",
                "foreignField": "id",
                "as": "task",
            },
        },
        doc! {
            "$match": {
                "task.0": { "$exists": true },
                "task.quest_id": QUEST_ID,
            },
        },
        doc! {
            "$project": {
                "_id": 0,
                "task_id": 1,
            },
        },
    ];

    let completed_tasks = collection.aggregate(pipeline, None).await;
    match completed_tasks {
        Ok(mut tasks_cursor) => {
            let signer = LocalWallet::from(SigningKey::from_secret_scalar(
                state.conf.nft_contract.private_key,
            ));

            let mut rewards = vec![];
            while let Some(result) = tasks_cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(task_id) = document.get_i32("task_id") {
                            if task_id != 1 && task_id <= 4 {
                                match get_nft(QUEST_ID, &query.addr, task_id as u32 - 1, &signer)
                                    .await
                                {
                                    Ok((token_id, sig)) => {
                                        rewards.push(Reward {
                                            task_id: task_id as u32,
                                            nft_contract: state.conf.nft_contract.address.clone(),
                                            token_id: token_id.to_string(),
                                            sig: (sig.r, sig.s),
                                        });
                                    }
                                    Err(_) => continue,
                                }
                            }
                        }
                    }
                    _ => continue,
                }
            }

            if rewards.is_empty() {
                get_error("No rewards found for this user".into())
            } else {
                (StatusCode::OK, Json(RewardResponse { rewards })).into_response()
            }
        }
        Err(_) => get_error("Error querying rewards".into()),
    }
}
