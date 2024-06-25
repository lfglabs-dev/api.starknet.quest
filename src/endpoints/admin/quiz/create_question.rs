use crate::models::{JWTClaims, QuestDocument, QuestTaskDocument, QuizInsertDocument, QuizQuestionDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateQuizQuestion {
    quiz_id: i64,
    question: String,
    options:Vec<String>,
    correct_answers: Vec<i64>,
});

#[route(
    post,
    "/admin/tasks/quiz/question/create",
    crate::endpoints::admin::quiz::create_question
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateQuizQuestion>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quiz_questions_collection = state
        .db
        .collection::<QuizQuestionDocument>("quiz_questions");
    let quests_collection = state.db.collection::<QuestDocument>("quests");
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");


    let pipeline = doc! {
        "quiz_name": &body.quiz_id,
    };
    let res = &tasks_collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // get the quest id
    let quest_id = res.as_ref().unwrap().id as i64;

    let res = verify_quest_auth(user, &quests_collection, &quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    // filter to get existing quiz
    let filter = doc! {
        "id": &body.quiz_id,
    };

    let existing_quiz = &quiz_collection
        .find_one(filter.clone(), None)
        .await
        .unwrap();
    if existing_quiz.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_question_doc = &quiz_questions_collection
        .find_one(last_id_filter.clone(), options.clone())
        .await
        .unwrap();

    let mut next_quiz_question_id = 1;
    if let Some(doc) = last_quiz_question_doc {
        let last_id = doc.id;
        next_quiz_question_id = last_id + 1;
    }

    let new_quiz_document = QuizQuestionDocument {
        quiz_id: body.quiz_id.clone(),
        question: body.question.clone(),
        options: body.options.clone(),
        correct_answers: body.correct_answers.clone(),
        id: next_quiz_question_id,
        kind: "text_choice".to_string(),
        layout: "default".to_string(),
    };

    return match quiz_questions_collection
        .insert_one(new_quiz_document, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => return get_error("Error creating task".to_string()),
    };
}
