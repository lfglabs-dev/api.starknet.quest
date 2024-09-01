// src/middleware.rs

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::config;

#[derive(Debug, Deserialize)]
pub struct JWTClaims {
    sub: String,
}

pub async fn auth_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, (StatusCode, String)> {
    let headers = req.headers();
    let conf = config::load();
    let secret_key = &conf.auth.secret_key; 

    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        let mut parts = auth_header.split_whitespace();
        if let Some("Bearer") = parts.next() {
            if let Some(token) = parts.next() {
                match decode::<JWTClaims>(
                    token,
                    &DecodingKey::from_secret(secret_key.as_bytes()),
                    &Validation::new(jsonwebtoken::Algorithm::HS256),
                ) {
                    Ok(_token_data) => Ok(next.run(req).await),
                    Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token was provided".to_string())),
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "Missing token was provided".to_string()))
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header format".to_string()))
        }
    } else {
        Err((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))
    }
}
