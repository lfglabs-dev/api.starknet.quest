use crate::models::{JWTClaims, QuestInsertDocument};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use mongodb::bson::{doc, from_document};
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateQuestQuery {
    name: String,
    desc: String,
    start_time: i64,
    expiry: Option<i64>,
    disabled: bool,
    category: String,
    logo: String,
    rewards_img: String,
    rewards_title: String,
    img_card: String,
    title_card: String,
    issuer: Option<String>,
}

async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateQuestQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestInsertDocument>("quests");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let nft_reward = doc! {
        "img": body.img_card.clone(),
        "level": 1,
    };

    let issuer = body.issuer.as_deref().unwrap_or_else(|| headers.get("user").unwrap().to_str().unwrap());

    let mut new_document = doc! {
        "name": &body.name,
        "desc": &body.desc,
        "disabled": &body.disabled,
        "start_time": &body.start_time,
        "id": &next_id,
        "category": &body.category,
        "issuer": issuer,
        "rewards_endpoint": "/quests/claimable",
        "rewards_title": &body.rewards_title,
        "rewards_img": &body.rewards_img,
        "rewards_nfts": vec![nft_reward],
        "logo": &body.logo,
        "img_card": &body.img_card,
        "title_card": &body.title_card,
    };

    if let Some(expiry) = &body.expiry {
        new_document.insert("expiry", expiry);
    } else {
        new_document.insert("expiry", None::<String>);
    }

    new_document.insert("experience", if issuer == "Starknet ID" { 50 } else { 10 });

    // Insert document to quest collection
    match collection
        .insert_one(from_document::<QuestInsertDocument>(new_document).unwrap(), None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"id": format!("{}", next_id)})),
        )
            .into_response(),
        Err(_) => get_error("Error creating quest".to_string()),
    }
}

// Export the router function
pub fn create_quest_router() -> Router {
    Router::new().route("/create_quest", post(handler))
}
