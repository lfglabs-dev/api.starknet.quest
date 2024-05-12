use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, Document};
use mongodb::options::{FindOneAndUpdateOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::QuestTaskDocument;

pub_struct!(Deserialize; UpdateTwitterFw {
    name: Option<String>,
    desc: Option<String>,
    username: Option<String>,
    id: i32,
});

#[route(put, "/admin/tasks/twitter_fw/update", crate::endpoints::admin::twitter::update_twitter_fw)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateTwitterFw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    // filter to get existing boost
    let filter = doc! {
        "id": &body.id,
    };
    let existing_task = &collection.find_one(filter.clone(), None).await.unwrap();

    // create a boost if it does not exist
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
        update_doc.insert("verify_redirect",format!("https://twitter.com/intent/user?screen_name={:?}", &username));
        update_doc.insert("href",format!("https://twitter.com/{:?}", &username));
    }

    // update boost
    let update = doc! {
        "$set": update_doc
    };
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
        Err(_e) => get_error("error updating boost".to_string()),
    };
}
