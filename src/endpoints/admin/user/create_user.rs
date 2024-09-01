use crate::models::{JWTClaims, LoginDetails};
use crate::utils::calculate_hash;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use axum_auto_routes::route;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; CreateCustom {
    user: String,
    password: String,
});

#[route(post, "/admin/user/create")]
pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;

    if user != "super_user" {
        return get_error("Operation not allowed with your account".to_string());
    };

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

pub fn create_user_routes() -> Router {
    Router::new().route("/create_user", post(handler))
}
