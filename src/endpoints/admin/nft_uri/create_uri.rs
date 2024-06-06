use crate::models::{NFTUri};
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
    image: String,
});

#[route(post, "/admin/nft_uri/create", crate::endpoints::admin::nft_uri::create_uri)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<NFTUri>("nft_uri");
    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = NFTUri {
        name: body.name.clone(),
        description: body.desc.clone(),
        image: body.image.clone(),
        quest_id : body.quest_id.clone() as i64,
        id: next_id,
        attributes: None,
    };

    // insert document to boost collection
    return match collection
        .insert_one(
            new_document,
            None,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Uri created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating boosts".to_string()),
    };
}
