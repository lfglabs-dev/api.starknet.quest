use crate::middleware::auth::auth_middleware;
use crate::models::QuestTaskDocument;
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
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

#[route(post, "/admin/tasks/twitter_rw/update", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<UpdateTwitterRw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_task_auth(sub, &collection, &body.id).await;
    if !res {
        return get_error("Error updating tasks".to_string());
    }

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
    if let Some(post_link) = &body.post_link {
        update_doc.insert("verify_redirect", &post_link);
        update_doc.insert("href", &post_link);
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
        Err(_e) => get_error("error updating task".to_string()),
    };
}
