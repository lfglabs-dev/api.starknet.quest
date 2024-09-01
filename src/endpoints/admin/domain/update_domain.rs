use crate::models::QuestTaskDocument;
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::routing::post;
use crate::models::JWTClaims;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;
use axum::Router;
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse, 
};

use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateDomainTask {
    name: Option<String>,
    desc: Option<String>,
    id: i32,
}

// Define the route handler
async fn update_domain_task_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<UpdateDomainTask>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_task_auth(user, &collection, &body.id).await;
    if !res {
        return get_error("Error updating tasks".to_string());
    }

    // Filter to get the existing task
    let filter = doc! {
        "id": body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }

    // Update task query
    let update = doc! {
        "$set": update_doc
    };

    // Update the document in the collection
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
pub fn update_domain_router() -> Router {
    Router::new().route("/update_domain", post(update_domain_task_handler))
}
