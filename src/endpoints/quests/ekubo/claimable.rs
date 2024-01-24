use crate::models::{AppState, CompletedTaskDocument, Reward, RewardResponse};
use crate::utils::{get_error, get_nft};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use starknet::{
    core::types::FieldElement,
    signers::{LocalWallet, SigningKey},
};
use std::sync::Arc;

const QUEST_ID: u32 = 9;
const TASK_IDS: &[u32] = &[37, 38, 39];
const LAST_TASK: u32 = TASK_IDS[2];
const NFT_LEVEL: u32 = 14;

#[derive(Deserialize)]
pub struct ClaimableQuery {
    addr: FieldElement,
}

#[route(
    get,
    "/quests/ekubo/claimable",
    crate::endpoints::quests::ekubo::claimable
)]
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

            let Ok((token_id, sig)) =
                get_nft(QUEST_ID, LAST_TASK, &query.addr, NFT_LEVEL, &signer).await
            else {
                return get_error("Signature failed".into());
            };

            rewards.push(Reward {
                task_id: LAST_TASK,
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
        Err(_) => get_error("Error querying rewards".into()),
    }
}
