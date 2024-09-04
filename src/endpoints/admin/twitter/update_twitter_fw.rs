use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    extract::State
};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateTwitterFw {
    name: Option<String>,
    desc: Option<String>,
    username: Option<String>,
    id: i32,
});

pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateTwitterFw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

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
