use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
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

    let new_document = doc! {
        "name": &body.name,
        "desc": &body.desc,
        "href": &body.invite_link,
        "quest_id" : &body.quest_id,
        "id": next_id,
        "cta": "Join now!",
        "verify_endpoint": "quests/discord_fw_callback",
        "verify_endpoint_type": "oauth_discord",
        "task_type":"discord",
        "discord_guild_id": &body.guild_id,
    };

    // insert document to boost collection
    return match collection
        .insert_one(
            from_document::<QuestTaskDocument>(new_document).unwrap(),
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
