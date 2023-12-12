use crate::models::BoostTable;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::from_document;
use std::sync::Arc;

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");
    let mut boosts = match collection.find(doc! {"hidden":false}, None).await {
        Ok(cursor) => cursor,
        Err(_) => return get_error("Error querying boosts".to_string()),
    };
    let mut boosts_array: Vec<BoostTable> = Vec::new();
    while let Some(result) = boosts.next().await {
        match result {
            Ok(document) => {
                boosts_array.push(document.into());
            }
            _ => continue,
        }
    }
    (StatusCode::OK, Json(boosts_array)).into_response()
}
