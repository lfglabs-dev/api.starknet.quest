use std::sync::Arc;

use crate::{
    common::verify_quiz::verify_quiz,
    models::{AppState, VerifyQuizQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde_json::json;
use starknet::core::types::FieldElement;
use crate::models::QuestTaskDocument;

#[route(post, "/quests/verify_quiz", crate::endpoints::quests::verify_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<VerifyQuizQuery>,
) -> impl IntoResponse {
    if body.addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let pipeline = vec![
        doc! {
            "$match": doc! {
                "quiz_name": &body.quiz_name
            }
        },
    ];

    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let task_id = match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut id = 0;
            while let Some(result) = cursor.try_next().await.unwrap() {
                id = result.get("id").unwrap().as_i64().unwrap();
            }
            id as u32
        }
        Err(_) => return get_error("Quiz name does not match".to_string()),
    };

    let user_answers_numbers: Result<Vec<Vec<usize>>, _> = body
        .user_answers_list
        .iter()
        .map(|inner_list| {
            inner_list
                .iter()
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
        })
        .collect();

    match user_answers_numbers {
        Ok(responses) => match verify_quiz(&state.db, body.addr, &body.quiz_name, &responses).await {
            true => match state.upsert_completed_task(body.addr, task_id).await {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            },
            false => get_error("Incorrect answers".to_string()),
        },
        Err(e) => get_error(format!("{}", e)),
    }
}
