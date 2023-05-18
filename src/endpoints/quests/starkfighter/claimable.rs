use crate::{models::AppState, utils::get_error};
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
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::{LocalWallet, Signer, SigningKey},
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
pub struct CompletedTaskDocument {
    task_id: u32,
    address: String,
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

#[derive(Serialize)]
pub struct QueryError {
    error: String,
}

#[derive(Serialize)]
pub struct Reward {
    task_id: u32,
    nft_contract: String,
    token_id: String,
    sig: (FieldElement, FieldElement), // Assume that the Signature is serialized as a String
}

#[derive(Serialize)]
pub struct RewardResponse {
    rewards: Vec<Reward>,
}

async fn get_nft(
    addr: &FieldElement,
    task_id: u32,
    signer: &LocalWallet,
) -> Result<(u32, Signature), Box<dyn std::error::Error + Send + Sync>> {
    let nft_level = match task_id {
        2 => 1,
        3 => 2,
        4 => 3,
        _ => {
            return Ok((
                0,
                Signature {
                    r: FieldElement::ZERO,
                    s: FieldElement::ZERO,
                },
            ))
        }
    };

    let token_id = nft_level + 100 * (rand::random::<u32>() % (2u32.pow(16)));
    let hashed = pedersen_hash(
        &pedersen_hash(
            &pedersen_hash(
                &pedersen_hash(&FieldElement::from(token_id), &FieldElement::ZERO),
                &FieldElement::from(QUEST_ID),
            ),
            &FieldElement::from(task_id),
        ),
        addr,
    );
    let sig = signer.sign_hash(&hashed).await?;
    Ok((token_id, sig))
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
                            if task_id != 1 {
                                match get_nft(&query.addr, task_id as u32, &signer).await {
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
