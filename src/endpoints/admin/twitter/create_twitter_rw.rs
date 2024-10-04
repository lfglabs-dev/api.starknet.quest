use crate::middleware::auth::auth_middleware;
use crate::models::{QuestDocument, QuestTaskDocument};
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
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateTwitterRw {
    name: String,
    desc: String,
    post_link: String,
    quest_id: i64,
});

#[route(post, "/admin/tasks/twitter_rw/create", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateTwitterRw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
   

    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(sub, &quests_collection, &body.quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let state_last_id = state.last_task_id.lock().await;

    let next_id = get_next_task_id(&collection, state_last_id.clone()).await;


    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        total_amount: None,
        verify_redirect: Some(body.post_link.clone()),
        href: body.post_link.clone(),
        quest_id: body.quest_id.clone(),
        id: next_id,
        verify_endpoint: "quests/verify_twitter_rw".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("twitter_rw".to_string()),
        cta: "Retweet".to_string(),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
        api_url: None,
        regex: None,
        calls: None,
    };

    // insert document to boost collection
    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating task".to_string()),
    };
}
