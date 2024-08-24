use crate::models::{QuizInsertDocument,JWTClaims};
use crate::{
    models::{AppState},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use axum::http::HeaderMap;


#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: i64,
}

#[route(
    get,
    "/admin/quiz/get_quiz"
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
    headers: HeaderMap
) -> impl IntoResponse {
    let _user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref());
    let collection = state.db.collection::<QuizInsertDocument>("quizzes");
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "id": query.id
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quiz_questions",
                "localField": "id",
                "foreignField": "quiz_id",
                "as": "questions"
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "questions._id": 0
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        return (StatusCode::OK, Json(document)).into_response();
                    }
                    _ => continue,
                }
            }
            get_error("Quiz not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
