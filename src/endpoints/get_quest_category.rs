use crate::{
    models::{AppState, QuestCategoryDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    name: String,
}

#[route(get, "/get_quest_category")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state
        .db
        .collection::<QuestCategoryDocument>("quest_categories");
    let filter = doc! {
        "name": &query.name
    };

    match collection.find_one(filter, None).await {
        Ok(option) => match option {
            Some(category) => (StatusCode::OK, Json(category)).into_response(),
            None => get_error("Category not found".to_string()),
        },
        Err(_) => get_error("Error querying quest category data".to_string()),
    }
}
