use crate::models::NFTUri;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Router,
    routing::post,
};
use crate::models::JWTClaims;

use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

#[derive(Deserialize)]
pub struct CreateCustom {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    image: Option<String>,
}

// Define the route handler
async fn update_nft_uri_handler(
    Extension(state): Extension<Arc<AppState>>, // Use Extension to extract state
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<NFTUri>("nft_uri");

    // Filter to get existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("description", desc);
    }
    if let Some(image) = &body.image {
        update_doc.insert("image", image);
    }

    // Update quest query
    let update = doc! {
        "$set": update_doc
    };

    // Insert document to boost collection
    match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
        .into_response(),
        Err(_) => get_error("Error updating tasks".to_string()),
    }
}

// Define the router for this module
pub fn update_nft_uri_router() -> Router {
    Router::new().route("/update_nft_uri", post(update_nft_uri_handler))
}
