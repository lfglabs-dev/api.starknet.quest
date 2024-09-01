use crate::models::{QuestDocument, QuestTaskDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::routing::post;
use crate::models::JWTClaims;
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Router,
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

#[derive(Deserialize)]
pub struct CreateDomainTask {
    name: String,
    desc: String,
    quest_id: i64,
}

// Define the route handler
async fn create_domain_task_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<CreateDomainTask>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &body.quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        total_amount: None,
        href: "https://app.starknet.id/".to_string(),
        quest_id: body.quest_id,
        id: next_id,
        verify_endpoint: "quests/verify_domain".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("domain".to_string()),
        cta: "Register a domain".to_string(),
        discord_guild_id: None,
        quiz_name: None,
        verify_redirect: None,
        contracts: None,
    };

    // Insert document into the collection
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
pub fn create_domain_router() -> Router {
    Router::new().route("/create_domain", post(create_domain_task_handler))
}
