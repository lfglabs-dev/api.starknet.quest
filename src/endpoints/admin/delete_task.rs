use crate::models::QuestTaskDocument;

use crate::{models::AppState, utils::get_error};
use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; DeleteTask {
   id: i32,
});

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<DeleteTask>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let filter = doc! {
        "id": body.id,
    };
    
     match collection.delete_one(filter, None).await {
        Ok(result) => {
            if result.deleted_count > 0 {
                (
                    StatusCode::OK,
                    Json(json!({"message": "deleted successfully"})),
                ).into_response()
            } else {
                get_error("Task does not exist".to_string())
            }
        },
        Err(_) => get_error("Error deleting task".to_string()),
    }
}

pub fn delete_task_routes() -> Router {
    Router::new().route("/remove_task", post(handler))
}
