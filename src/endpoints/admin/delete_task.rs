use crate::models::QuestTaskDocument;

use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json}
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; DeleteTask {
   id: i32,
});

pub async fn handler(
    State(state): State<Arc<AppState>>,
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