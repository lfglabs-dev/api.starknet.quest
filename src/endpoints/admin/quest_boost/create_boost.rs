use crate::models::{BoostTable, QuestDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateBoostQuery {
    amount: i32,
    token: String,
    num_of_winners: i64,
    token_decimals: i64,
    name: String,
    quest_id: i32,
    hidden: bool,
    expiry: i64,
    img_url: String,
}

async fn create_boost_handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateBoostQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = BoostTable {
        name: body.name.clone(),
        amount: body.amount.clone(),
        token_decimals: body.token_decimals.clone(),
        token: body.token.clone(),
        expiry: body.expiry.clone(),
        num_of_winners: body.num_of_winners.clone(),
        quests: vec![body.quest_id.clone()],
        id: next_id,
        hidden: body.hidden.clone(),
        img_url: body.img_url.clone(),
        winner: None,
    };

    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        ).into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    }
}

pub fn create_boost_router() -> Router {
    Router::new().route("/create_quest_boost", post(create_boost_handler))
}
