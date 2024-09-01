use crate::models::{JWTClaims, QuestTaskDocument};
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

#[derive(Deserialize)]
pub struct UpdateDiscordTask {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    invite_link: Option<String>,
    guild_id: Option<String>,
}

// Define the route handler
async fn update_discord_task_handler(
    Extension(state): Extension<Arc<AppState>>, // Extract state using Extension
    headers: HeaderMap,
    body: Json<UpdateDiscordTask>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

   
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
    if let Some(invite_link) = &body.invite_link {
        update_doc.insert("href", invite_link);
    }
    if let Some(guild_id) = &body.guild_id {
        update_doc.insert("discord_guild_id", guild_id);
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
pub fn update_discord_router() -> Router {
    Router::new().route("/update_discord", post(update_discord_task_handler))
}
