use crate::models::{
    JWTClaims, QuestDocument, QuestTaskDocument, QuizInsertDocument, QuizQuestionDocument,
};
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
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneAndUpdateOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateQuiz {
    id:u32,
    quiz_id:u32,
    question: Option<String>,
    options:Option<Vec<String>>,
    correct_answers: Option<Vec<i64>>,
});

#[route(
    post,
    "/admin/tasks/quiz/question/update",
    crate::endpoints::admin::quiz::update_question
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;

    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");

    let quests_collection = state.db.collection::<QuestDocument>("quests");
    let quiz_questions_collection = state
        .db
        .collection::<QuizQuestionDocument>("quiz_questions");

    let pipeline = doc! {
        "quiz_name": &body.quiz_id,
    };
    let res = &tasks_collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // get the quest id
    let quest_id = res.as_ref().unwrap().id as i32;

    let res = verify_quest_auth(user, &quests_collection, &quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    // filter to get existing quiz
    let filter = doc! {
        "id": &body.quiz_id,
    };
    let existing_task = &quiz_collection
        .find_one(filter.clone(), None)
        .await
        .unwrap();

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

    // update question
    let update = doc! {
        "$set": update_doc,
    };
    let options = FindOneAndUpdateOptions::default();
    return match quiz_questions_collection
        .find_one_and_update(question_filter, update.clone(), options)
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
