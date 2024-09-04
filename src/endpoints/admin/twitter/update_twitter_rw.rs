use crate::models::QuestTaskDocument;
use crate::utils::get_error;
use crate::models::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateTwitterRw {
    name: Option<String>,
    desc: Option<String>,
    post_link: Option<String>,
    id: i32,
});

pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateTwitterRw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let filter = doc! { "id": body.id };
    let existing_task = collection.find_one(filter.clone(), None).await.unwrap();

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
    if let Some(post_link) = &body.post_link {
        update_doc.insert("verify_redirect", post_link);
        update_doc.insert("href", post_link);
    }

    let update = doc! { "$set": update_doc };
    let options = FindOneAndUpdateOptions::default();

    match collection
        .find_one_and_update(filter, update, options)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Updated successfully"})),
        ).into_response(),
        Err(_e) => get_error("Error updating task".to_string()),
    }
}
