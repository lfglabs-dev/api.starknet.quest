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
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]

pub struct GetCompletedQuestsQuery {
    addr: FieldElement,
}


/*
 this endpoint will return static data for one address
 -> get total users
 Steps to get data over time:
 1) iterate over one week timestamps and add total points and get top 3 and get user position
 2) iterate over one month timestamps and add total points and get top 3 and get user position
 3) iterate over one year timestamps and add total points and get top 3 and get user position
 */

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    println!("{}", address);
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "address": address
            }
        },

    ];
    let tasks_collection = state.db.collection::<Document>("user_exp");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<u32> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                quests.push(result.get("experience").unwrap().as_i32().unwrap() as u32);
            }
            print!("{:?}", quests);
            (StatusCode::OK, Json(quests)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
