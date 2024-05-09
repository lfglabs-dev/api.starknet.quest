use crate::models::BoostTable;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use std::sync::Arc;

#[route(get, "/boost/get_boosts", crate::endpoints::quest_boost::get_boosts)]
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
