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

pub_struct!(Deserialize; UpdateQuiz {
    id:u32,
    question: Option<String>,
    options:Option<Vec<String>>,
    correct_answers: Option<String>,
});

#[route(post, "/admin/tasks/quiz/question/update", crate::endpoints::admin::quiz::update_question)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");

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

    if let Some(question) = &body.question {
        update_doc.insert("question", question);
    }
    if let Some(options) = &body.options {
        update_doc.insert("options", options);
    }
    if let Some(correct_answers) = &body.correct_answers {
        update_doc.insert("correct_answers", correct_answers);
    }

    // update question
    let update = doc! {
        "$set": update_doc,
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
        Err(_e) => get_error("error updating boost".to_string()),
    };
}
