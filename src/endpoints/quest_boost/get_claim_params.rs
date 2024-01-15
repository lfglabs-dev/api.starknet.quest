use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::str::FromStr;

use mongodb::bson::{doc, Document};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::core::crypto::ecdsa_sign;
use starknet::core::{crypto::pedersen_hash, types::FieldElement};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClaimBoostQuery {
    boost_id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetClaimBoostQuery>,
) -> impl IntoResponse {
    let boost_id = query.boost_id;
    let collection = state.db.collection::<Document>("boosts");
    let res = collection.find_one(doc! {"id":boost_id}, None).await.unwrap();

    // if no boost found with the requested id
    if res.is_none() {
        return get_error(format!("Boost with id {} not found", boost_id));
    }

    let boost: Document = res.unwrap();
    let num_of_winners = boost.get("num_of_winners").unwrap().as_i32().unwrap();
    let amount = boost.get("amount").unwrap().as_i32().unwrap() as u32 / num_of_winners as u32;
    let token = boost.get("token").unwrap().as_str().unwrap();
    let address = boost.get("winner").unwrap().as_str().unwrap();

    let hashed = pedersen_hash(
        &FieldElement::from(boost_id),
        &pedersen_hash(
            &FieldElement::from(amount),
            &pedersen_hash(
                &FieldElement::from(0 as u32),
                &pedersen_hash(
                    &FieldElement::from_str(token).unwrap(),
                    &FieldElement::from_str(address).unwrap(),
                ),
            ),
        ),
    );

    match ecdsa_sign(&state.conf.quest_boost.private_key, &hashed) {
        Ok(signature) => (
            StatusCode::OK,
            Json(json!({"address": address, "r": signature.r, "s": signature.s})),
        )
            .into_response(),
        Err(e) => get_error(format!("Error while generating signature: {}", e)),
    }
}