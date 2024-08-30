use axum::{
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JWTClaims {
    sub: String, // You can add more fields as needed
}

pub async fn auth_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, (StatusCode, String)> {
    let headers = req.headers();
    let secret_key = b"your_secret_key"; // Replace with the actual secret key (sorry for this)

    match headers.get(AUTHORIZATION) {
        Some(auth_header) => {
            let auth_str = auth_header.to_str().unwrap_or("");
            let mut parts = auth_str.split_whitespace();

            if let Some("Bearer") = parts.next() {
                if let Some(token) = parts.next() {
                    match decode::<JWTClaims>(
                        token,
                        &DecodingKey::from_secret(secret_key),
                        &Validation::new(jsonwebtoken::Algorithm::HS256),
                    ) {
                        Ok(_token_data) => {
                            // Token is valid, then i proceed to the next middleware || handler
                            Ok(next.run(req).await)
                        }
                        Err(_) => {
                            // Token is invalid -> Throw error
                            Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
                        }
                    }
                } else {
                    // Missing token after "Bearer" -> take note.
                    Err((StatusCode::UNAUTHORIZED, "Missing token".to_string()))
                }
            } else {
                // Incorrect authorization header format
                Err((
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header format".to_string(),
                ))
            }
        }
        None => {
            // Authorization header is missing
            Err((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))
        }
    }
}
