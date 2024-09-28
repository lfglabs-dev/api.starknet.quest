use crate::models::{QuestDocument, QuestTaskDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use crate::middleware::auth::auth_middleware;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json}
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;
use std::str::FromStr;
use std::sync::Arc;

pub_struct!(Deserialize; CreateCustomAPI {
    quest_id: i64,
    name: String,
    desc: String,
    href: String,
    cta: String,
    api_url: String,
    regex: String,
});

#[route(post, "/admin/tasks/custom_api/create", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<CreateCustomAPI>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(sub, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    // Build a vector of FieldElement from the comma separated contracts string
    let parsed_contracts: Vec<FieldElement> = body
        .contracts
        .split(",")
        .map(|x| FieldElement::from_str(&x).unwrap())
        .collect();

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        verify_redirect: None,
        href: body.href.clone(),
        total_amount: None,
        quest_id: body.quest_id,
        id: next_id,
        cta: body.cta.clone(),
        verify_endpoint: "quests/verify_custom_api".to_string(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("custom_api".to_string()),
        discord_guild_id: None,
        quiz_name: None,
        contracts: None,
        api_url: Some(body.api_url.clone()),
        regex: Some(body.regex.clone()),
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
