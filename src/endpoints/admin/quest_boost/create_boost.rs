use crate::models::{BoostTable};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use mongodb::bson::{doc, from_document};

#[derive(Deserialize)]
pub struct CreateBoostQuery {
    amount: Option<i32>,
    token: Option<String>,
    num_of_winners: Option<i64>,
    token_decimals: Option<i64>,
    expiry: Option<i64>,
    name: Option<String>,
    img_url: Option<String>,
    quest_id: i32,
}

#[route(post, "/admin/quest_boost/create_boost", crate::endpoints::admin::quest_boost::create_boost)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateBoostQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = doc! {
            "name": &body.name,
            "img_url": &body.img_url,
            "amount": &body.amount,
            "token_decimals": &body.token_decimals,
            "token":&body.token,
            "expiry": &body.expiry,
            "num_of_winners": &body.num_of_winners,
            "quests": [&body.quest_id],
            "id": next_id,
            "hidden": false,
        };

    // insert document to boost collection
    return match collection
        .insert_one(from_document::<BoostTable>(new_document).unwrap(), None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
