use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorMessage {
    error: String,
}

pub fn get_error(error: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
}
