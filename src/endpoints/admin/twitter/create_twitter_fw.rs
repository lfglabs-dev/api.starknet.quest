use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::{ FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::QuestTaskDocument;

pub_struct!(Deserialize; CreateTwitterFw {
    name: String,
    desc: String,
    username: String,
    quest_id: i32,
});

#[route(post, "/admin/tasks/twitter_fw/create", crate::endpoints::admin::twitter::create_twitter_fw)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateTwitterFw>,
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
            "verify_redirect": format!("https://twitter.com/intent/user?screen_name={}", &body.username),
            "href": format!("https://twitter.com/{}", &body.username),
            "quest_id" : &body.quest_id,
            "id": next_id,
            "verify_endpoint": "quests/verify_twitter_fw",
            "verify_endpoint_type": "default",
            "type": "twitter_fw",
            "cta": "Follow",
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
