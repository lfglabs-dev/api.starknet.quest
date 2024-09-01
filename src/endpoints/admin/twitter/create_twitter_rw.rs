use crate::models::QuestTaskDocument;
use crate::models::AppState;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Extension, Router,
};
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

async fn create_twitter_retweet_task(
    Extension(state): Extension<Arc<AppState>>,
    body: Json<CreateTwitterRw>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();
    
    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        total_amount: None,
        verify_redirect: Some(body.post_link.clone()),
        href: body.post_link.clone(),
        quest_id: body.quest_id,
        id: next_id,
        verify_endpoint: "quests/verify_twitter_rw".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("twitter_rw".to_string()),
        cta: "Retweet".to_string(),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
    };

    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Error creating task"})).into_response(),
        ),
    }
}

pub fn create_twitter_rw_router() -> Router {
    Router::new().route("/create_twitter_rw", post(create_twitter_retweet_task))
}
