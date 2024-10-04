use crate::middleware::auth::auth_middleware;
use crate::models::{BoostTable, QuestDocument, QuestTaskDocument};
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

#[derive(Deserialize)]
pub struct CreateBoostQuery {
    amount: i32,
    token: String,
    num_of_winners: i64,
    token_decimals: i64,
    name: String,
    quest_id: i32,
    hidden: bool,
    expiry: i64,
    img_url: String,
}

#[route(post, "/admin/quest_boost/create_boost", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateBoostQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");
    let quests_collection = state.db.collection::<QuestDocument>("quests");
    let insert_collection = state.db.collection::<QuestTaskDocument>("quests");

    let res = verify_quest_auth(sub, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating boost".to_string());
    };

    let state_last_id = state.last_task_id.lock().await;

    let next_id = get_next_task_id(&insert_collection, state_last_id.clone()).await;

    let new_document = BoostTable {
        name: body.name.clone(),
        amount: body.amount.clone(),
        token_decimals: body.token_decimals.clone(),
        token: body.token.clone(),
        expiry: body.expiry.clone(),
        num_of_winners: body.num_of_winners.clone(),
        quests: vec![body.quest_id.clone()],
        id: next_id,
        hidden: body.hidden.clone(),
        img_url: body.img_url.clone(),
        winner: None,
    };

    // insert document to boost collection
    return match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
