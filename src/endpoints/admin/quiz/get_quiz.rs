use crate::{
    models::{AppState, QuizInsertDocument},
    utils::get_error,
};
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: i64,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
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
                    Err(_) => continue,
                }
            }
            get_error("Quiz not found".to_string())
        }
        Err(_) => get_error("Error querying quiz".to_string()),
    }
}

pub fn get_quiz_routes() -> Router {
    Router::new().route("/get_quiz", get(handler))
}
