use crate::middleware::auth::auth_middleware;
use crate::models::{NFTUri, QuestDocument, QuestTaskDocument};
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

pub_struct!(Deserialize; CreateCustom {
    quest_id: i64,
    name: String,
    desc: String,
    image: String,
});

#[route(post, "/admin/nft_uri/create", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<NFTUri>("nft_uri");
    let quests_collection = state.db.collection::<QuestDocument>("quests");
    let insert_collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_quest_auth(sub, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating task".to_string());
    };


    let state_last_id = state.last_task_id.lock().await;

    let next_id = get_next_task_id(&insert_collection, state_last_id.clone()).await;

    let new_document = NFTUri {
        name: body.name.clone(),
        description: body.desc.clone(),
        image: body.image.clone(),
        quest_id: body.quest_id.clone() as i64,
        id: next_id.into(),
        attributes: None,
    };

    // insert document to boost collection
    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Uri created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
