use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::{FindOneAndUpdateOptions};
use serde_json::json;
use std::sync::Arc;
use mongodb::bson::Document;
use serde::Deserialize;
use crate::models::QuestTaskDocument;
use axum::http::HeaderMap;
use crate::utils::verify_task_auth;


pub_struct!(Deserialize; UpdateQuiz {
    id:u32,
    name: Option<String>,
    desc: Option<String>,
    help_link: Option<String>,
    cta: Option<String>,
});

#[route(post, "/admin/tasks/quiz/update", crate::endpoints::admin::quiz::update_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref())  as String;
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");


    let res= verify_task_auth(user,  &tasks_collection,&(body.id as i32)).await;
    if !res{
        return get_error("Error updating tasks".to_string());
    }


    // filter to get existing boost
    let filter = doc! {
        "id": &body.id,
    };
    let existing_task = &tasks_collection.find_one(filter.clone(), None).await.unwrap();

    // create a quiz if it does not exist
    if existing_task.is_none() {
        return get_error("No quiz found".to_string());
    }


    let mut update_doc = Document::new();

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(href) = &body.help_link {
        update_doc.insert("href", href);
    }
    if let Some(cta) = &body.cta {
        update_doc.insert("cta", cta);
    }

    // update boost
    let update = doc! {
        "$set": update_doc
    };
    let options = FindOneAndUpdateOptions::default();
    return match tasks_collection
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
