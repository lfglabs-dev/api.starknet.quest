use crate::{
    models::{AppState, JWTClaims, QuestDocument},
    utils::get_error,
};
use axum::http::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::{doc, from_document};
use std::sync::Arc;

#[route(
    get,
    "/admin/quest/get_quests"
)]
pub async fn handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref());
    let mut pipeline = vec![];
    if user != "super_user" {
        pipeline.push(doc! {
            "$match": doc! {
                "issuer":user
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
