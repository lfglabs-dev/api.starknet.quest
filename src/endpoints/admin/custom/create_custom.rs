use axum::response::IntoResponse;
use axum::{routing::post, Router};
use crate::models::{AppState, QuestTaskDocument};
use crate::utils::get_error;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use axum::extract::{Json, Extension};
use axum::http::StatusCode;
use std::sync::Arc;

// Define the request body structure
#[derive(Deserialize)]
pub struct CreateCustom {
    quest_id: i64,
    name: String,
    desc: String,
    cta: String,
    href: String,
    api: String,
}

// Define the route handler
async fn create_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    body: Json<CreateCustom>,
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

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        verify_redirect: Some(body.href.clone()),
        href: body.href.clone(),
        quest_id: body.quest_id,
        total_amount: None,
        id: next_id,
        cta: body.cta.clone(),
        verify_endpoint: body.api.clone(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("custom".to_string()),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
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
pub fn create_custom_router() -> Router {
    Router::new().route("/task", post(create_handler))
}
