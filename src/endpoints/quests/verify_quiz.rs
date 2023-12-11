use std::sync::Arc;

use crate::{
    common::verify_quiz::verify_quiz,
    models::{AppState, VerifyQuizQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use starknet::core::types::FieldElement;

fn get_task_id(quiz_name: &str) -> Option<u32> {
    match quiz_name {
        "carmine" => Some(40),
        "morphine" => Some(42),
        "zklend" => Some(53),
        "avnu" => Some(54),
        "sithswap" => Some(55),
        "starknetid" => Some(56),
        "gigabrain_1" => Some(51),
        "gigabrain_2" => Some(57),
        "gigabrain_3" => Some(58),
        "aa_mastery_1" => Some(52),
        "aa_mastery_2" => Some(59),
        "aa_mastery_3" => Some(60),
        "focustree" => Some(61),
        "element" => Some(64),
        "briq" => Some(67),
        "element_starknetid" => Some(73),
        "nostra" => Some(79),
        "rango" => Some(95),
        "braavos" => Some(98),
        _ => None,
    }
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<VerifyQuizQuery>,
) -> impl IntoResponse {
    if body.addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let task_id = match get_task_id(&body.quiz_name) {
        Some(id) => id,
        None => return get_error("Quiz name does not match".to_string()),
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
