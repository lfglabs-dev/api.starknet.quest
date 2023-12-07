use crate::{
    models::{AppState},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use mongodb::bson::{doc, Document};
use serde::Deserialize;
use std::sync::Arc;
use serde_json::json;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    boost_id: u32,
    status: bool,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let boost_id = query.boost_id;
    let status = query.status;
    let collection = state.db.collection::<Document>("boosts");
    let res = collection.find_one(doc! {"id":boost_id}, None).await.unwrap();

    // if no boost found with the requested id
    if res.is_none() {
        return get_error(format!("Boost with id {} not found", boost_id));
    }

    // update boost with claimed field equal to true
    let update = doc! {"$set": {"claimed": status}};
    collection.update_one(doc! {"id":boost_id}, update, None).await.unwrap();
    (StatusCode::OK, Json(json!({"res": true}))).into_response()
}
