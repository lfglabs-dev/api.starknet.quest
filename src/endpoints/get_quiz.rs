use crate::{config::QuizQuestionType, models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuizQuery {
    id: String,
    // addr could be used as entropy for sending a server side randomized order
    // let's keep on client side for now
    #[allow(dead_code)]
    addr: FieldElement,
}

pub_struct!(Clone, Serialize; QuizQuestionResp {
    kind: String,
    layout: String,
    question: String,
    options: Vec<String>,
    image_for_layout: Option<String>
});

#[derive(Clone, Serialize)]
pub struct QuizResponse {
    name: String,
    desc: String,
    questions: Vec<QuizQuestionResp>,
}

#[route(get, "/get_quiz", crate::endpoints::get_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuizQuery>,
) -> impl IntoResponse {
    let quizzes_from_config = &state.conf.quizzes;
    println!("{:?}", query.id);
    match quizzes_from_config.get(&query.id) {
        Some(quiz) => {
            let questions: Vec<QuizQuestionResp> = quiz
                .questions
                .iter()
                .map(|question| QuizQuestionResp {
                    kind: match question.kind {
                        QuizQuestionType::TextChoice => "text_choice".to_string(),
                        QuizQuestionType::ImageChoice => "image_choice".to_string(),
                        QuizQuestionType::Ordering => "ordering".to_string(),
                    },
                    layout: question.layout.clone(),
                    question: question.question.clone(),
                    options: question.options.clone(),
                    image_for_layout: question.image_for_layout.clone(),
                })
                .collect();
            let quiz_response = QuizResponse {
                name: quiz.name.clone(),
                desc: quiz.desc.clone(),
                questions,
            };

            (StatusCode::OK, Json(quiz_response)).into_response()
        }
        None => get_error("Quiz not found".to_string()),
    }
}
