use crate::middleware::auth::auth_middleware;
use crate::models::{QuestDocument, QuestTaskDocument, QuizInsertDocument, QuizQuestionDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; DeleteQuiz {
    id: u32,
    quiz_id: u32,
});

#[route(post, "/admin/tasks/quiz/question/delete", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    body: Json<DeleteQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let quests_collection = state.db.collection::<QuestDocument>("quests");
    let quiz_questions_collection = state
        .db
        .collection::<QuizQuestionDocument>("quiz_questions");

    // quiz exists?
    let pipeline = doc! {
        "quiz_name": &body.quiz_id,
    };
    let res = &tasks_collection.find_one(pipeline, None).await.unwrap();
    if res.is_none() {
        return get_error("quiz does not exist".to_string());
    }

    // get quest id and verify auth
    let quest_id = res.as_ref().unwrap().id as i64;
    let res = verify_quest_auth(sub, &quests_collection, &quest_id).await;
    if !res {
        return get_error("Error deleting question".to_string());
    };

    // quiz exists?
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

    // delete the question
    let question_filter = doc! {
        "id": &body.id,
    };

    return match quiz_questions_collection.delete_one(question_filter, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "deleted successfully"})),
        )
            .into_response(),
        Err(_e) => get_error("error deleting question".to_string()),
    };
}