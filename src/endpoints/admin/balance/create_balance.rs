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
use starknet::core::types::FieldElement;
use std::str::FromStr;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

// Define the request body structure
#[derive(Deserialize)]
pub struct CreateBalance {
    quest_id: i64,
    name: String,
    desc: String,
    contracts: String,
    href: String,
    cta: String,
}

// Define the route handler
async fn create_balance_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<CreateBalance>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    // Build a vector of FieldElement from the comma-separated contracts string
    let parsed_contracts: Vec<FieldElement> = body
        .contracts
        .split(",")
        .map(|x| FieldElement::from_str(&x).unwrap())
        .collect();

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        verify_redirect: None,
        href: body.href.clone(),
        total_amount: None,
        quest_id: body.quest_id,
        id: next_id,
        cta: body.cta.clone(),
        verify_endpoint: "quests/verify_balance".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("balance".to_string()),
        discord_guild_id: None,
        quiz_name: None,
        contracts: Some(parsed_contracts),
    };

    // Insert document into collection
    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_) => get_error("Error creating tasks".to_string()),
    }
}

// Define the router for this module
pub fn create_balance_router() -> Router {
    Router::new().route("/create_balance", post(create_balance_handler))
}
