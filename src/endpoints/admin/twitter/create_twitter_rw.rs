use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::QuestTaskDocument;

pub_struct!(Deserialize; CreateTwitterRw {
    name: String,
    desc: String,
    post_link: String,
    quest_id: i32,
});

#[route(post, "/admin/tasks/twitter_rw/create", crate::endpoints::admin::twitter::create_twitter_rw)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateTwitterRw>,
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
            "verify_redirect": &body.post_link,
            "href": &body.post_link,
            "quest_id" : &body.quest_id,
            "id": next_id,
            "verify_endpoint": "quests/verify_twitter_rw",
            "verify_endpoint_type": "default",
            "task_type": "twitter_rw",
            "cta": "Retweet",
        };

    // insert document to boost collection
    return match collection
        .insert_one(from_document::<QuestTaskDocument>(new_document).unwrap(), None)
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
