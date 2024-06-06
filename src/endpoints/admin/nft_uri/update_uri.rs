use crate::models::{NFTUri,JWTClaims};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use axum::http::HeaderMap;
use jsonwebtoken::{Validation,Algorithm,decode,DecodingKey};


pub_struct!(Deserialize; CreateCustom {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    image: Option<String>,
});

#[route(post, "/admin/nft_uri/update", crate::endpoints::admin::nft_uri::update_uri)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let _user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<NFTUri>("nft_uri");

    // filter to get existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("description", desc);
    }
    if let Some(image) = &body.image {
        update_doc.insert("image", image);
    }

    // update quest query
    let update = doc! {
        "$set": update_doc
    };


    // insert document to boost collection
    return match collection
        .find_one_and_update(filter, update, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error updating tasks".to_string()),
    };
}
