use crate::middleware::auth::auth_middleware;
use crate::models::{QuestInsertDocument, QuestTaskDocument};
use crate::utils::get_next_task_id;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateQuestQuery {
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
});

#[route(post, "/admin/quest/create", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateQuestQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestInsertDocument>("quests");
    let insert_collection = state.db.collection::<QuestTaskDocument>("tasks");
   
    let state_last_id = state.last_task_id.lock().await;

    let next_id = get_next_task_id(&insert_collection, state_last_id.clone()).await;

    let nft_reward = doc! {
        "img": body.img_card.clone().to_string(),
        "level": 1,
    };

    let issuer = match sub == "super_user" {
        true => {
            let result_issuer = (&body.issuer).as_ref().unwrap();
            result_issuer
        }
        false => &sub,
    };

    let mut new_document = doc! {
        "name": &body.name,
        "desc": &body.desc,
        "disabled": &body.disabled,
        "start_time": &body.start_time,
        "id": &next_id,
        "category":&body.category,
        "issuer": &issuer,
        "rewards_endpoint":"/quests/claimable",
        "rewards_title": &body.rewards_title,
        "rewards_img": &body.rewards_img,
        "rewards_nfts": vec![nft_reward],
        "logo": &body.logo,
        "img_card": &body.img_card,
        "title_card": &body.title_card,
    };

    match &body.expiry {
        Some(expiry) => new_document.insert("expiry", expiry),
        None => new_document.insert("expiry", None::<String>),
    };

    match issuer == "Starknet ID" {
        true => new_document.insert("experience", 50),
        false => new_document.insert("experience", 10),
    };

    // insert document to boost collection
    return match collection
        .insert_one(
            from_document::<QuestInsertDocument>(new_document).unwrap(),
            None,
        )
        .await
    {
        Ok(_res) => {
            return (
                StatusCode::OK,
                Json(json!({"id": format!("{}",&next_id)})).into_response(),
            )
                .into_response();
        }
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
