use crate::config::Config;
use crate::utils::get_error;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct TokenURI {
    name: String,
    description: String,
    image: String,
    attributes: Option<Vec<Attribute>>,
}

#[derive(Serialize)]
pub struct Attribute {
    trait_type: String,
    value: Value,
}

#[derive(Serialize)]
pub enum Value {
    String(String),
    Number(i32),
    Array(Vec<String>),
}

#[derive(Deserialize)]
pub struct LevelQuery {
    level: Option<String>,
}

pub async fn handler(
    Extension(state): Extension<Arc<Config>>,
    Query(level_query): Query<LevelQuery>,
) -> Response {
    let level = level_query
        .level
        .and_then(|level_str| level_str.parse::<i32>().ok());

    fn get_level(level_int: i32) -> &'static str {
        match level_int {
            2 => "Silver",
            3 => "Gold",
            _ => "Bronze",
        }
    }

    match level {
        Some(level_int) if level_int > 0 && level_int <= 3 => {
            let image_link = format!(
                "{}/starkfighter/level{}.webp",
                state.variables.app_link, level_int
            );
            let response = TokenURI {
                name: format!("StarkFighter {} Arcade", get_level(level_int)),
                description: "A starknet.quest NFT won during the Starkfighter event.".into(),
                image: image_link,
                attributes: Some(vec![Attribute {
                    trait_type: "level".into(),
                    value: Value::Number(level_int),
                }]),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        _ => get_error("Error, this level is not correct".into()),
    }
}
