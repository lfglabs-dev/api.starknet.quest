use crate::middleware::auth::auth_middleware;
use crate::models::{Call, QuestDocument, QuestTaskDocument};
use crate::utils::get_next_task_id;
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;

use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateContract {
    quest_id: i64,
    name: String,
    desc: String,
    href: String,
    cta: String,
    calls: Vec<Call>
});

#[route(post, "/admin/tasks/contract/create", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateContract>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(sub, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let state_last_id = state.last_task_id.lock().await;

    let next_id = get_next_task_id(&collection, state_last_id.clone()).await;

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        verify_redirect: None,
        href: body.href.clone(),
        total_amount: None,
        quest_id: body.quest_id,
        id: next_id,
        cta: body.cta.clone(),
        verify_endpoint: "quests/verify_contract".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("contract".to_string()),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
        calls: Some(body.calls),
        api_url: None,
        regex: None,
    };

    // insert document to boost collection
    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating tasks".to_string()),
    };
}
