use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse, 
};

use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateDomainTask {
    name: Option<String>,
    desc: Option<String>,
    id: i32,
}

// Define the route handler
pub async fn handler(
    State(state): State<Arc<AppState>>, // Extract state using Extension
    Json(body): Json<UpdateDomainTask>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    // Filter to get the existing task
    let filter = doc! {
        "id": body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }

    // Update task query
    let update = doc! {
        "$set": update_doc
    };

    // Update the document in the collection
    match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_) => get_error("Error updating tasks".to_string()),
    }
}

