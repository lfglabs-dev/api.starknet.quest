use crate::models::{QuestDocument, JWTClaims, QuestInsertDocument};
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
use axum::http::HeaderMap;
use serde::Deserialize;
use mongodb::bson::{doc, from_document};
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};
use crate::endpoints::get_quests::NFTItem;

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
});

#[route(post, "/admin/quest/create", crate::endpoints::admin::quest::create_quest)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateQuestQuery>,
    // headers: HeaderMap,
) -> impl IntoResponse {
    let user = "admin";
    // let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref());
    let collection = state.db.collection::<QuestInsertDocument>("quests");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let nft_reward = doc! {
        "img": body.rewards_img.clone().to_string(),
        "level": 1,
    };

    let mut new_document = doc! {
        "name": &body.name,
        "desc": &body.desc,
        "disabled": &body.disabled,
        "start_time": &body.start_time,
        "id": next_id,
        "category":&body.category,
        "issuer": &user,
        "rewards_endpoint":"/quests/claimable",
        "rewards_title": &body.rewards_title,
        "rewards_img": &body.rewards_img,
        "rewards_nfts": vec![nft_reward],
        "logo": &body.logo,
        "img_card": &body.img_card,
        "title_card": &body.title_card,
    };

    match &body.expiry {
        Some(expiry) =>
            new_document.insert("expiry", expiry),
        None => new_document.insert("expiry", None::<String>),
    };

    match user == "admin" {
        true =>
            new_document.insert("experience", 50),
        false => new_document.insert("experience", 10),
    };


    // insert document to boost collection
    return match collection
        .insert_one(from_document::<QuestInsertDocument>(new_document).unwrap(), None)
        .await
    {
        Ok(res) => {
            println!("Quest created successfully {:?}", res);
            return (
                StatusCode::OK,
                Json(json!({"message": "Quest created successfully"})).into_response(),
            )
                .into_response();
        }
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
