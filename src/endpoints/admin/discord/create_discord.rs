use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateCustom {
    quest_id: u32,
    name: String,
    desc: String,
    invite_link: String,
    guild_id: String,
});

#[route(post, "/admin/tasks/discord/create", crate::endpoints::admin::discord::create_discord)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        href: body.invite_link.clone(),
        quest_id: body.quest_id.clone(),
        id: next_id,
        cta: "Join now!".to_string(),
        verify_endpoint: "quests/discord_fw_callback".to_string(),
        verify_endpoint_type: "oauth_discord".to_string(),
        task_type: Some("discord".to_string()),
        discord_guild_id: Some(body.guild_id.clone()),
        quiz_name: None,
        verify_redirect: None,
    };

    // insert document to boost collection
    return match collection
        .insert_one(new_document,
            None,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
