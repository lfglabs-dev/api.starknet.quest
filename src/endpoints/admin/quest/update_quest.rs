use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use mongodb::bson::{doc, Document};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::models::JWTClaims;
use jsonwebtoken::decode;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;

#[derive(Deserialize)]
pub struct UpdateQuestQuery {
    id: i32,
    name: Option<String>,
    desc: Option<String>,
    start_time: Option<i64>,
    expiry: Option<i64>,
    disabled: Option<bool>,
    category: Option<String>,
    logo: Option<String>,
    rewards_img: Option<String>,
    rewards_title: Option<String>,
    img_card: Option<String>,
    title_card: Option<String>,
    issuer: Option<String>,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateQuestQuery>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestDocument>("quests");

    let mut filter = doc! {
        "id": body.id,
    };

    if user != "super_user" {
        filter.insert("issuer", user);
    }

    let existing_quest = collection.find_one(filter.clone(), None).await.unwrap();
    if existing_quest.is_none() {
        return get_error("quest does not exist".to_string());
    }

    let mut update_doc = Document::new();

    if let Some(name) = body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(expiry) = body.expiry {
        update_doc.insert("expiry", expiry);
    }
    if let Some(start_time) = body.start_time {
        update_doc.insert("start_time", start_time);
    }
    if let Some(disabled) = body.disabled {
        update_doc.insert("disabled", disabled);
    }
    if let Some(category) = body.category {
        update_doc.insert("category", category);
    }
    if let Some(logo) = body.logo {
        update_doc.insert("logo", logo);
    }
    if let Some(issuer) = body.issuer {
        update_doc.insert("issuer", issuer);
    }
    if let Some(ref rewards_img) = body.rewards_img {
        update_doc.insert("rewards_img", rewards_img.clone()); // Use clone() to avoid moving the value

        let nft_reward = doc! {
            "img": rewards_img.clone(),  // Use clone() here as well
            "level": 1,
        };
        update_doc.insert("rewards_nfts", vec![nft_reward]);
    }

    if let Some(rewards_title) = body.rewards_title {
        update_doc.insert("rewards_title", rewards_title);
    }
    if let Some(img_card) = body.img_card {
        update_doc.insert("img_card", img_card);
    }
    if let Some(title_card) = body.title_card {
        update_doc.insert("title_card", title_card);
    }

    let update = doc! {
        "$set": update_doc
    };

    match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        )
            .into_response(),
        Err(_) => get_error("error updating quest".to_string()),
    }
}

pub fn update_quest_routes() -> Router {
    Router::new().route("/update_quest", post(handler))
}
