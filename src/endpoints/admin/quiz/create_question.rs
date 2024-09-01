use crate::{
    models::{
        AppState, JWTClaims, QuestDocument, QuestTaskDocument, QuizInsertDocument, QuizQuestionDocument,
    },
    utils::get_error,
};
use axum::{
    extract::{Json, Extension},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::post,
    Router,
};
use mongodb::{
    bson::doc,
    options::FindOneOptions,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateQuizQuestion {
    quiz_id: i64,
    question: String,
    options: Vec<String>,
    correct_answers: Vec<i64>,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateQuizQuestion>,
) -> impl IntoResponse {
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quiz_questions_collection = state.db.collection::<QuizQuestionDocument>("quiz_questions");
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");

    let pipeline = doc! {
        "quiz_name": body.quiz_id,
    };
    let res = tasks_collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("quiz does not exist".to_string());
    }


    // filter to get existing quiz
    let filter = doc! {
        "id": body.quiz_id,
    };

    let existing_quiz = quiz_collection.find_one(filter.clone(), None).await.unwrap();
    if existing_quiz.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_question_doc = quiz_questions_collection.find_one(last_id_filter, options).await.unwrap();

    let next_quiz_question_id = if let Some(doc) = last_quiz_question_doc {
        doc.id + 1
    } else {
        1
    };

    let new_quiz_document = QuizQuestionDocument {
        quiz_id: body.quiz_id,
        question: body.question,
        options: body.options,
        correct_answers: body.correct_answers,
        id: next_quiz_question_id,
        kind: "text_choice".to_string(),
        layout: "default".to_string(),
    };

    match quiz_questions_collection.insert_one(new_quiz_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})),
        ).into_response(),
        Err(_) => get_error("Error creating task".to_string()),
    }
}

pub fn create_question_routes() -> Router {
    Router::new().route("/create_question", post(handler))
}
