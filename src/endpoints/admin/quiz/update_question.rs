use crate::models::{
    JWTClaims, QuestDocument, QuestTaskDocument, QuizInsertDocument, QuizQuestionDocument,
};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
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
    question: Option<String>,
    options: Option<Vec<String>>,
    correct_answers: Option<Vec<i64>>,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quiz_questions_collection = state.db.collection::<QuizQuestionDocument>("quiz_questions");

    let pipeline = doc! {
        "quiz_name": &body.quiz_id,
    };
    let res = &tasks_collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    let filter = doc! {
        "id": &body.quiz_id,
    };
    let existing_task = &quiz_collection.find_one(filter.clone(), None).await.unwrap();
    if existing_task.is_none() {
        return get_error("No quiz found".to_string());
    }

    let mut update_doc = Document::new();

    if let Some(question) = &body.question {
        update_doc.insert("question", question);
    }
    if let Some(options) = &body.options {
        update_doc.insert("options", options);
    }
    if let Some(correct_answers) = &body.correct_answers {
        update_doc.insert("correct_answers", correct_answers);
    }

    let question_filter = doc! {
        "id": &body.id,
    };

    let update = doc! {
        "$set": update_doc,
    };
    let options = FindOneAndUpdateOptions::default();
    match quiz_questions_collection.find_one_and_update(question_filter, update.clone(), options).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        ).into_response(),
        Err(_e) => get_error("error updating task".to_string()),
    }
}

pub fn update_question_routes() -> Router {
    Router::new().route("/update_question", post(handler))
}
