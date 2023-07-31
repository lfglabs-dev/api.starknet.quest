use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    match collection
        .find_one(doc! {"id": query.id, "disabled" : false}, None)
        .await
    {
        Ok(Some(quest)) => (StatusCode::OK, Json(quest)).into_response(),
        Ok(None) => get_error("Quest not found".to_string()),
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
