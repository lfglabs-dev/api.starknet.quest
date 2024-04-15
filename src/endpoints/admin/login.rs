use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::from_document;
use serde::Deserialize;
use std::sync::Arc;
use chrono::Utc;
use serde_json::json;
use crate::models::{JWTClaims, LoginDetails};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    code: String,
}

#[route(get, "/login", crate::endpoints::admin::login)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<LoginDetails>("login_details");
    let pipeline = [
        doc! {
            "$match": {
                "code": query.code,
            }
        },
    ];


    let mut validation = Validation::new(Algorithm::HS256);

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        let secret_key = &state.conf.auth.secret_key;
                        if let Ok(mut login) = from_document::<LoginDetails>(document) {
                            let new_exp = (Utc::now().timestamp_millis() + &state.conf.auth.expiry_duration) as usize;
                            let user_claims = JWTClaims {
                                sub: login.user.parse().unwrap(),
                                exp: new_exp,
                            };
                            let token = encode(&Header::default(), &user_claims, &EncodingKey::from_secret(&secret_key.as_ref())).unwrap();
                            let decoded_token = decode::<JWTClaims>(&token, &DecodingKey::from_secret(&secret_key.as_ref()), &validation).unwrap();
                            return (StatusCode::OK, Json(json!({"token":token}))).into_response();
                        }
                    }
                    Err(e) => {
                        return get_error(e.to_string());
                    }
                }
            }
            get_error("Incorrect Password".to_string())
        }
        Err(e) => {
            return get_error(e.to_string());
        }
    }
}
