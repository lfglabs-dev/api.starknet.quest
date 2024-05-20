use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::{QuizInsertDocument, QuizQuestionDocument};

pub_struct!(Deserialize; CreateQuizQuestion {
    quiz_id: i64,
    question: String,
    options:Vec<String>,
    correct_answers: Vec<i64>,
});

#[route(post, "/admin/tasks/quiz/question/create", crate::endpoints::admin::quiz::create_question)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateQuizQuestion>,
) -> impl IntoResponse {
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quiz_questions_collection = state.db.collection::<QuizQuestionDocument>("quiz_questions");

    // filter to get existing quiz
    let filter = doc! {
        "id": &body.quiz_id,
    };

    let existing_quiz = &quiz_collection.find_one(filter.clone(), None).await.unwrap();
    if existing_quiz.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_question_doc = &quiz_questions_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();

    let mut next_quiz_question_id = 1;
    if let Some(doc) = last_quiz_question_doc {
        let last_id = doc.id;
        next_quiz_question_id = last_id + 1;
    }

    let new_quiz_document = doc! {
            "quiz_id": &body.quiz_id,
            "question": &body.question,
            "options": &body.options,
            "correct_answers": &body.correct_answers,
            "id": next_quiz_question_id,
            "kind": "text_choice",
            "layout": "default"
    };

    return match quiz_questions_collection
        .insert_one(from_document::<QuizQuestionDocument>(new_quiz_document).unwrap(), None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => return get_error("Error creating quiz".to_string()),
    };
}
