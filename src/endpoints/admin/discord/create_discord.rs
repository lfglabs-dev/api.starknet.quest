use crate::models::{JWTClaims, QuestDocument, QuestTaskDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::{routing::post, Router};
use axum::extract::{Json, Extension};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

// Define the request body structure
#[derive(Deserialize)]
pub struct CreateCustom {
    quest_id: i64,
    name: String,
    desc: String,
    invite_link: String,
    guild_id: String,
}

// Define the route handler
async fn create_discord_task_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref());
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating task".to_string());
    }

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        href: body.invite_link.clone(),
        quest_id: body.quest_id,
        id: next_id,
        total_amount: None,
        cta: "Join now!".to_string(),
        verify_endpoint: "quests/discord_fw_callback".to_string(),
        verify_endpoint_type: "oauth_discord".to_string(),
        task_type: Some("discord".to_string()),
        discord_guild_id: Some(body.guild_id.clone()),
        quiz_name: None,
        verify_redirect: None,
        contracts: None,
    };

    // Insert document into collection
    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_) => get_error("Error creating task".to_string()),
    }
}

// Define the router for this module
pub fn create_discord_router() -> Router {
    Router::new().route("/tasks", post(create_discord_task_handler))
}
