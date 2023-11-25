use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use starknet::{
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::LocalWallet,
};
use starknet::signers::SigningKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClaimBoostQuery {
    boost_id: i64,
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetClaimBoostQuery>,
) -> impl IntoResponse {

    // const CLAIM_STRING: &str ="Claim Boost Reward";

    // get boost id from params
    let boost_id = query.boost_id as u32;
    let address = query.addr;

    // make a signature for the boost id, amount, token, user address
    //
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        state.conf.nft_contract.private_key,
    ));

    // has with boost id, CLAIM_STRING, address
    let hashed = pedersen_hash(
        &FieldElement::from(boost_id),
        &FieldElement::from(address),
    );

    println!("hashed: {:?}", hashed);
    let sig = signer.sign_hash(&hashed).await?;
    // return the signature

    get_error("Error querying quests".to_string())

    // let address = query.addr.to_string();
    // let pipeline = vec![
    //     doc! {
    //         "$match": doc! {
    //             "address": address
    //         }
    //     },
    //     doc! {
    //         "$lookup": doc! {
    //             "from": "tasks",
    //             "localField": "task_id",
    //             "foreignField": "id",
    //             "as": "associatedTask"
    //         }
    //     },
    //     doc! {
    //         "$unwind": "$associatedTask"
    //     },
    //     doc! {
    //         "$group": doc! {
    //             "_id": "$associatedTask.quest_id",
    //             "done": doc! {
    //                 "$sum": 1
    //             }
    //         }
    //     },
    //     doc! {
    //         "$lookup": doc! {
    //             "from": "tasks",
    //             "localField": "_id",
    //             "foreignField": "quest_id",
    //             "as": "tasks"
    //         }
    //     },
    //     doc! {
    //         "$match": doc! {
    //             "$expr": doc! {
    //                 "$eq": [
    //                     "$done",
    //                     doc! {
    //                         "$size": "$tasks"
    //                     }
    //                 ]
    //             }
    //         }
    //     },
    //     doc! {
    //         "$project": doc! {
    //             "quest_id": "$_id",
    //             "_id": 0
    //         }
    //     },
    // ];
    // let tasks_collection = state.db.collection::<Document>("completed_tasks");
    // match tasks_collection.aggregate(pipeline, None).await {
    //     Ok(mut cursor) => {
    //         let mut quests: Vec<u32> = Vec::new();
    //         while let Some(result) = cursor.try_next().await.unwrap() {
    //             quests.push(result.get("quest_id").unwrap().as_i32().unwrap() as u32);
    //         }
    //         (StatusCode::OK, Json(quests)).into_response()
    //     }
    //     Err(_) => get_error("Error querying quests".to_string()),
    // }
}
