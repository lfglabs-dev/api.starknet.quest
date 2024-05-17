use crate::{
    models::{AppState, QuestDocument, JWTClaims},
    utils::get_error,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use std::sync::Arc;
use axum::http::HeaderMap;
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};

#[route(get, "/admin/get_quests", crate::endpoints::admin::quest::get_quests)]
pub async fn handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref());
    let collection = state.db.collection::<QuestDocument>("quests");
    println!("User: {:?}", user);
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "issuer":user
            }
        },
    ];

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
