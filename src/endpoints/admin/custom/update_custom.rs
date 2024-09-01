use crate::models::JWTClaims;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;
use axum::response::IntoResponse;
use axum::{routing::post, Router};
use crate::models::{AppState, QuestTaskDocument};
use crate::utils::{get_error, verify_task_auth};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use axum::extract::{Json, Extension};
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

// Define the request body structure
#[derive(Deserialize)]
pub struct UpdateCustom {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    cta: Option<String>,
    verify_endpoint: Option<String>,
    verify_endpoint_type: Option<String>,
    verify_redirect: Option<String>,
    href: Option<String>,
}

// Define the route handler
async fn update_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<UpdateCustom>,
) -> impl IntoResponse {
   let collection = state.db.collection::<QuestTaskDocument>("tasks");

    // Filter to get the existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(href) = &body.href {
        update_doc.insert("href", href);
    }
    if let Some(cta) = &body.cta {
        update_doc.insert("cta", cta);
    }
    if let Some(verify_redirect) = &body.verify_redirect {
        update_doc.insert("verify_redirect", verify_redirect);
    }
    if let Some(verify_endpoint) = &body.verify_endpoint {
        update_doc.insert("verify_endpoint", verify_endpoint);
    }
    if let Some(verify_endpoint_type) = &body.verify_endpoint_type {
        update_doc.insert("verify_endpoint_type", verify_endpoint_type);
    }

    // Update quest query
    let update = doc! {
        "$set": update_doc
    };

    // Update document in the collection
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
pub fn update_custom_router() -> Router {
    Router::new().route("/update_custom", post(update_handler))
}
