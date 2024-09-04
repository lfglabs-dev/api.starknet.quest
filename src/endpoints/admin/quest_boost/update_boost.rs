use crate::models::BoostTable;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json}
};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateBoostQuery {
    id: i32,
    amount: Option<i32>,
    token: Option<String>,
    num_of_winners: Option<i64>,
    token_decimals: Option<i64>,
    expiry: Option<i64>,
    name: Option<String>,
    img_url: Option<String>,
    hidden: Option<bool>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateBoostQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    // Verify if the boost exists
    let filter = doc! { "id": &body.id };
    let existing_boost = collection.find_one(filter.clone(), None).await.unwrap();

    if existing_boost.is_none() {
        return get_error("Boost does not exist".to_string());
    }

    let mut update_doc = Document::new();

    if let Some(amount) = &body.amount {
        update_doc.insert("amount", amount);
    }
    if let Some(token) = &body.token {
        update_doc.insert("token", token);
    }
    if let Some(expiry) = &body.expiry {
        update_doc.insert("expiry", expiry);
    }
    if let Some(num_of_winners) = &body.num_of_winners {
        update_doc.insert("num_of_winners", num_of_winners);
    }
    if let Some(token_decimals) = &body.token_decimals {
        update_doc.insert("token_decimals", token_decimals);
    }
    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(img_url) = &body.img_url {
        update_doc.insert("img_url", img_url);
    }
    if let Some(hidden) = &body.hidden {
        update_doc.insert("hidden", hidden);
    }

    let update = doc! { "$set": update_doc };
    let options = FindOneAndUpdateOptions::default();

    match collection.find_one_and_update(filter, update, options).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Updated successfully"})),
        )
            .into_response(),
        Err(_e) => get_error("Error updating boost".to_string()),
    }
}
