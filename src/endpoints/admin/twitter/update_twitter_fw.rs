use crate::models::{JWTClaims, QuestTaskDocument};
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Extension, Router,
};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

pub_struct!(Deserialize; UpdateTwitterFw {
    name: Option<String>,
    desc: Option<String>,
    username: Option<String>,
    id: i32,
});

async fn twitter_update_handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<UpdateTwitterFw>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_task_auth(user, &collection, &body.id).await;
    if !res {
        return get_error("Error updating tasks".to_string());
    }

    let filter = doc! { "id": &body.id };
    let existing_task = &collection.find_one(filter.clone(), None).await.unwrap();

    if existing_task.is_none() {
        return get_error("Task does not exist".to_string());
    }

    let mut update_doc = Document::new();

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(username) = &body.username {
        update_doc.insert(
            "verify_redirect",
            "https://twitter.com/intent/user?screen_name=".to_string() + username,
        );
        update_doc.insert("href", "https://twitter.com/".to_string() + username);
    }

    let update = doc! { "$set": update_doc };
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
        Err(_e) => get_error("error updating task".to_string()),
    };
}

pub fn update_twitter_fw_router() -> Router {
    Router::new().route("/update_twitter_fw", post(twitter_update_handler))
}
