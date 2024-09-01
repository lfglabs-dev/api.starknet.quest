use crate::{
    models::{AppState, QuestDocument, QuestTaskDocument, QuizInsertDocument},
    utils::get_error,
};
use axum::{
    extract::{Json, Extension},
    http::StatusCode,
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
pub struct CreateQuiz {
    name: String,
    desc: String,
    help_link: String,
    cta: String,
    intro: String,
    quest_id: i64,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(body): Json<CreateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    // Verify if the quest exists
    let quest_exists = quests_collection.find_one(doc! { "id": body.quest_id }, None).await.is_ok();
    if !quest_exists {
        return get_error("Quest does not exist".to_string());
    }

    // Get the last quiz ID in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_doc = quiz_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();
    let next_quiz_id = match last_quiz_doc {
        Some(doc) => doc.id + 1,
        None => 1,
    };

    let new_quiz_document = QuizInsertDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        id: next_quiz_id,
        intro: body.intro.clone(),
    };

    if quiz_collection.insert_one(new_quiz_document, None).await.is_err() {
        return get_error("Error creating quiz".to_string());
    }

    // Get the last task ID in increasing order
    let last_task_doc = tasks_collection.find_one(last_id_filter, options).await.unwrap();
    let next_task_id = match last_task_doc {
        Some(doc) => doc.id + 1,
        None => 1,
    };

    let new_task_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        href: body.help_link.clone(),
        total_amount: None,
        cta: body.cta.clone(),
        quest_id: body.quest_id,
        id: next_task_id,
        verify_endpoint: "/quests/verify_quiz".to_string(),
        verify_endpoint_type: "quiz".to_string(),
        quiz_name: Some(next_quiz_id as i64),
        task_type: Some("quiz".to_string()),
        discord_guild_id: None,
        verify_redirect: None,
        contracts: None,
    };

    match tasks_collection.insert_one(new_task_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"id": next_quiz_id})),
        ).into_response(),
        Err(_) => get_error("Error creating quiz".to_string()),
    }
}

pub fn create_quiz_routes() -> Router {
    Router::new().route("/create_quiz", post(handler))
}
