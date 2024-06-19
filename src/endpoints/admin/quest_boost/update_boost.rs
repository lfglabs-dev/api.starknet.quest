use crate::models::{BoostTable, JWTClaims, QuestDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateBoostQuery {
    id: i32,
    amount: Option<i32>,
    token: Option<String>,
    num_of_winners: Option<i64>,
    token_decimals: Option<i64>,
    expiry: Option<i64>,
    name: Option<String>,
    img_url: Option<String>,
    hidden: Option<bool>,
});

#[route(
post,
"/admin/quest_boost/update_boost",
crate::endpoints::admin::quest_boost::update_boost
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<UpdateBoostQuery>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<BoostTable>("boosts");
    let questcollection = state.db.collection::<QuestDocument>("quests");

    let pipeline = doc! {
            "id": &body.id,
    };

    let res = &collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("boost does not exist".to_string());
    }
    let quest_id = res.as_ref().unwrap().quests[0];
    let res = verify_quest_auth(user, &questcollection, &(quest_id as i32)).await;

    if !res {
        return get_error("Error updating boost".to_string());
    };

    // filter to get existing boost
    let filter = doc! {
        "id": &body.id,
    };

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

    // update boost
    let update = doc! {
        "$set": update_doc
    };
    let options = FindOneAndUpdateOptions::default();
    return match collection
        .find_one_and_update(filter, update, options)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        )
            .into_response(),
        Err(_e) => get_error("error updating boost".to_string()),
    };
}
