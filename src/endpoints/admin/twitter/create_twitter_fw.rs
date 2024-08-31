use crate::models::{JWTClaims, QuestDocument, QuestTaskDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Extension, Router,
};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

pub_struct!(Deserialize; CreateTwitterFw {
    name: String,
    desc: String,
    username: String,
    quest_id: i64,
});

async fn twitter_create_handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateTwitterFw>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &body.quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        total_amount: None,
        verify_redirect: Some(format!(
            "https://twitter.com/intent/user?screen_name={}",
            body.username.clone()
        )),
        href: format!("https://twitter.com/{}", body.username.clone()),
        quest_id: body.quest_id.clone(),
        id: next_id,
        verify_endpoint: "quests/verify_twitter_fw".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("twitter_fw".to_string()),
        cta: "Follow".to_string(),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
    };

    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating task".to_string()),
    };
}

pub fn create_twitter_router() -> Router {
    Router::new().route("/tasks", post(twitter_create_handler))
}
