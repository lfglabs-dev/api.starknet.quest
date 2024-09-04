use crate::models::LoginDetails;
use crate::utils::calculate_hash;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateCustom {
    user: String,
    password: String,
});

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateCustom>,
) -> impl IntoResponse {
    let collection = state.db.collection::<LoginDetails>("login_details");
    let hashed_password = calculate_hash(&body.password);

    let new_document = LoginDetails {
        user: body.user.clone(),
        code: hashed_password.to_string(),
    };

    match collection.insert_one(new_document, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "User added successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating user".to_string()),
    }
}
