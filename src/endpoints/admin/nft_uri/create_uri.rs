use crate::models::{NFTUri, QuestDocument};
use crate::utils::verify_quest_auth;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Router,
    routing::post,
};
use crate::models::JWTClaims;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

#[derive(Deserialize)]
pub struct CreateCustom {
    quest_id: i64,
    name: String,
    desc: String,
    image: String,
}

// Define the route handler
async fn create_nft_uri_handler(
    Extension(state): Extension<Arc<AppState>>, // Use Extension to extract state
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<NFTUri>("nft_uri");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    let res = verify_quest_auth(user, &quests_collection, &body.quest_id).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = NFTUri {
        name: body.name.clone(),
        description: body.desc.clone(),
        image: body.image.clone(),
        quest_id: body.quest_id as i64,
        id: next_id,
        attributes: None,
    };

    // Insert document into the collection
    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Uri created successfully"})).into_response(),
        )
        .into_response(),
        Err(_) => get_error("Error creating boosts".to_string()),
    }
}

// Define the router for this module
pub fn create_nft_uri_router() -> Router {
    Router::new().route("/nft_uri", post(create_nft_uri_handler))
}
