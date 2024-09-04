use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateDomainTask {
    name: String,
    desc: String,
    quest_id: i64,
}

// Define the route handler
pub async fn handler(
    State(state): State<Arc<AppState>>, // Extract state using Extension
    Json(body): Json<CreateDomainTask>,
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
