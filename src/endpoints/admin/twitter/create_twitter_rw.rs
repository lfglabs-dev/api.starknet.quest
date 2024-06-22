use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::{QuestDocument, QuestTaskDocument,JWTClaims};
use axum::http::HeaderMap;
use crate::utils::verify_quest_auth;
use jsonwebtoken::{Validation,Algorithm,decode,DecodingKey};


pub_struct!(Deserialize; CreateTwitterRw {
    name: String,
    desc: String,
    post_link: String,
    quest_id: i64,
});

#[route(post, "/admin/tasks/twitter_rw/create", crate::endpoints::admin::twitter::create_twitter_rw)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateTwitterRw>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let quests_collection = state.db.collection::<QuestDocument>("quests");


    let res= verify_quest_auth(user, &quests_collection, &body.quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
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
    };

    // insert document to boost collection
    return match collection
        .insert_one(new_document, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating task".to_string()),
    };
}
