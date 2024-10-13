use crate::middleware::auth::auth_middleware;
use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use std::sync::Arc;

#[route(get, "/admin/quest/get_quests", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
) -> impl IntoResponse {
    let mut pipeline = vec![];
    if sub != "super_user" {
        pipeline.push(doc! {
            "$match": doc! {
                "issuer":sub
            }
        });
    }
    let collection = state.db.collection::<QuestDocument>("quests");

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quests: Vec<QuestDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(mut quest) = from_document::<QuestDocument>(document) {
                            if let Some(expiry) = &quest.expiry {
                                quest.expiry_timestamp = Some(expiry.to_string());
                            }
                            quests.push(quest);
                        }
                    }
                    _ => continue,
                }
            }

            if quests.is_empty() {
                get_error("No quests found".to_string())
            } else {
                (StatusCode::OK, Json(quests)).into_response()
            }
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
