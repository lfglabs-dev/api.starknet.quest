use std::sync::Arc;

use crate::{
    common::verify_quiz::verify_quiz,
    models::{AppState, VerifyQuizQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use starknet::core::types::FieldElement;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<VerifyQuizQuery>,
) -> impl IntoResponse {
    let task_id = 56;
    if body.addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

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
        Ok(responses) => match verify_quiz(&state.conf, body.addr, &body.quiz_name, &responses) {
            true => match state.upsert_completed_task(body.addr, task_id).await {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            },
            false => get_error("Incorrect answers".to_string()),
        },
        Err(e) => get_error(format!("{}", e)),
    }
}
