use crate::models::{QuestTaskDocument, QuizInsertDocument};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router
};
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateQuiz {
    id: u32,
    quiz_id: u32,
    name: Option<String>,
    desc: Option<String>,
    help_link: Option<String>,
    cta: Option<String>,
    intro: Option<String>,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");

    // Check if the quiz exists
    let quiz_filter = doc! {
        "id": body.quiz_id,
    };
    let existing_quiz = quiz_collection.find_one(quiz_filter.clone(), None).await.unwrap();
    if existing_quiz.is_none() {
        return get_error("No quiz found".to_string());
    }

    // Update quiz
    let mut quiz_update_doc = Document::new();
    if let Some(name) = &body.name {
        quiz_update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        quiz_update_doc.insert("desc", desc);
    }
    if let Some(intro) = &body.intro {
        quiz_update_doc.insert("intro", intro);
    }

    let quiz_update = doc! {
        "$set": quiz_update_doc
    };
    let quiz_update_options = FindOneAndUpdateOptions::default();
    if let Err(_) = quiz_collection
        .find_one_and_update(quiz_filter, quiz_update, quiz_update_options)
        .await
    {
        return get_error("Error updating quiz".to_string());
    }

    // Update task
    let task_filter = doc! {
        "id": body.id,
    };
    let mut task_update_doc = Document::new();
    if let Some(name) = &body.name {
        task_update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        task_update_doc.insert("desc", desc);
    }
    if let Some(help_link) = &body.help_link {
        task_update_doc.insert("href", help_link);
    }
    if let Some(cta) = &body.cta {
        task_update_doc.insert("cta", cta);
    }

    let task_update = doc! {
        "$set": task_update_doc
    };
    let task_update_options = FindOneAndUpdateOptions::default();
    match tasks_collection
        .find_one_and_update(task_filter, task_update, task_update_options)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Updated successfully"})),
        ).into_response(),
        Err(_) => get_error("Error updating task".to_string()),
    }
}

pub fn update_quiz_routes() -> Router {
    Router::new().route("/update_quiz", post(handler))
}
