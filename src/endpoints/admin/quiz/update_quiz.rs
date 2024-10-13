use crate::middleware::auth::auth_middleware;
use crate::models::{QuestTaskDocument, QuizInsertDocument};
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateQuiz {
    id:u32,
    quiz_id:u32,
    name: Option<String>,
    desc: Option<String>,
    help_link: Option<String>,
    cta: Option<String>,
    intro: Option<String>,
});

#[route(post, "/admin/tasks/quiz/update", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");

    let res = verify_task_auth(sub, &tasks_collection, &(body.id as i32)).await;
    if !res {
        return get_error("Error updating tasks".to_string());
    }

    // filter to get existing quiz
    let filter = doc! {
        "id": &body.quiz_id,
    };
    let existing_quiz = &quiz_collection
        .find_one(filter.clone(), None)
        .await
        .unwrap();

    // create a quiz if it does not exist
    if existing_quiz.is_none() {
        return get_error("No quiz found".to_string());
    }

    let mut quiz_update_doc = Document::new();

    if let Some(name) = &body.name {
        quiz_update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        quiz_update_doc.insert("desc", desc);
    }
    if let Some(cta) = &body.intro {
        quiz_update_doc.insert("intro", cta);
    }

    // update quiz
    let update = doc! {
        "$set": quiz_update_doc
    };
    let options = FindOneAndUpdateOptions::default();
    match quiz_collection
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

    // update quiz
    let task_update = doc! {
        "$set": update_doc
    };
    let task_filter = doc! {
        "id": &body.id,
    };
    let options = FindOneAndUpdateOptions::default();
    return match tasks_collection
        .find_one_and_update(task_filter, task_update, options)
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
