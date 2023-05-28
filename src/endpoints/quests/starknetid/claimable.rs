use crate::models::{AppState, CompletedTaskDocument, Reward, RewardResponse};
use crate::utils::get_error;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use mongodb::bson::{doc, Bson};
use serde::Deserialize;
use starknet::{
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::{LocalWallet, Signer, SigningKey},
};
use std::sync::Arc;

const QUEST_ID: u32 = 1;
const TASK_IDS: &[u32] = &[5, 6, 7, 8];
const NFT_LEVEL: u32 = 4;

#[derive(Deserialize)]
pub struct ClaimableQuery {
    addr: FieldElement,
}

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
                "task_id": { "$in": TASK_IDS },
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
                "task.quest_id": QUEST_ID,
            },
        },
        doc! {
            "$group": {
                "_id": "$address",
                "completed_tasks": { "$push": "$task_id" },
            },
        },
        doc! {
            "$match": {
                "completed_tasks": { "$all": TASK_IDS },
            },
        },
    ];

    let completed_tasks = collection.aggregate(pipeline, None).await;
    match completed_tasks {
        Ok(mut tasks_cursor) => {
            if tasks_cursor.next().await.is_none() {
                return get_error("User hasn't completed all tasks".into());
            }

            let signer = LocalWallet::from(SigningKey::from_secret_scalar(
                state.conf.nft_contract.private_key,
            ));

            let mut rewards = vec![];
            for task_id in TASK_IDS {
                match get_nft(&query.addr, NFT_LEVEL, &signer).await {
                    Ok((token_id, sig)) => {
                        rewards.push(Reward {
                            task_id: *task_id,
                            nft_contract: state.conf.nft_contract.address.clone(),
                            token_id: token_id.to_string(),
                            sig: (sig.r, sig.s),
                        });
                    }
                    Err(_) => continue,
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

async fn get_nft(
    addr: &FieldElement,
    nft_level: u32,
    signer: &LocalWallet,
) -> Result<(u32, Signature), Box<dyn std::error::Error + Send + Sync>> {
    let token_id = nft_level + 100 * (rand::random::<u32>() % (2u32.pow(16)));
    let hashed = pedersen_hash(
        &pedersen_hash(
            &pedersen_hash(
                &pedersen_hash(&FieldElement::from(token_id), &FieldElement::ZERO),
                &FieldElement::from(QUEST_ID),
            ),
            &FieldElement::from(nft_level),
        ),
        addr,
    );
    let sig = signer.sign_hash(&hashed).await?;
    Ok((token_id, sig))
}
