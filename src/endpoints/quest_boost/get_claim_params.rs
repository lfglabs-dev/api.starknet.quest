use std::str::FromStr;
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
use serde_json::json;
use starknet::{
    core::{
        crypto::{pedersen_hash},
        types::FieldElement,
    },
};
use starknet::core::crypto::ecdsa_sign;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClaimBoostQuery {
    boost_id: u32,
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetClaimBoostQuery>,
) -> impl IntoResponse {
    let boost_id = query.boost_id;
    let address = query.addr;

    let pipeline = vec![
        doc! {
            "$match": {
                "id": boost_id
            },
        },
        doc! {
            "$project": {
                "_id": "0",
                "amount":"$amount",
                "token":"$token",
            },
        },
    ];

    let collection = state.db.collection::<Document>("boosts");
    let mut res = match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            cursor.try_next().await.unwrap()
        }
        Err(_) => return get_error("Error querying boosts".to_string()),
    };

    let boost: Document = res.unwrap();
    let amount = boost.get("amount").unwrap().as_i32().unwrap() as u32;
    let token = boost.get("token").unwrap().as_str().unwrap();

    let hashed = pedersen_hash(&FieldElement::from(boost_id),
                               &pedersen_hash(&FieldElement::from(amount),
                                              &pedersen_hash(&FieldElement::from_str(token).unwrap(),
                                                             &address)));

    let signature = ecdsa_sign(&state.conf.nft_contract.private_key, &hashed).unwrap();

    match ecdsa_sign(&state.conf.nft_contract.private_key, &hashed) {
        Ok(signature) => (
            StatusCode::OK,
            Json(
                json!({"address": address, "r": signature.r, "s": signature.s}),
            ),
        )
            .into_response(),
        Err(e) => get_error(format!("Error while generating signature: {}", e)),
    }
}
