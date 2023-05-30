use crate::models::AppState;
use crate::utils::get_error;
use axum::{
    extract::{Query, State},
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
    value: u32,
}

#[derive(Deserialize)]
pub struct UriQuery {
    nft_type: Option<String>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UriQuery>,
) -> Response {
    let nft_type = query
        .nft_type
        .and_then(|nft_type_str| nft_type_str.parse::<u32>().ok());

    fn get_arcade_level(nft_type_int: u32) -> &'static str {
        match nft_type_int {
            1 => "Bronze,",
            2 => "Silver",
            3 => "Gold",
            _ => "Error",
        }
    }

    match nft_type {
        Some(nft_type_int) if nft_type_int > 0 && nft_type_int <= 3 => {
            let image_link = format!(
                "{}/starkfighter/nft_type{}.webp",
                state.conf.variables.app_link, nft_type_int
            );
            let response = TokenURI {
                name: format!("StarkFighter {} Arcade", get_arcade_level(nft_type_int)),
                description: "A starknet.quest NFT won during the Starkfighter event.".into(),
                image: image_link,
                attributes: Some(vec![Attribute {
                    trait_type: "level".into(),
                    value: nft_type_int,
                }]),
            };
            (StatusCode::OK, Json(response)).into_response()
        }

        Some(4) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "StarknetID Totem".into(),
                description: "A Starknet Quest NFT won for creating a StarknetID profile.".into(),
                image: format!(
                    "{}/starkfighter/starknetid/nf2.webp",
                    state.conf.variables.app_link
                ),
                attributes: None,
            }),
        )
            .into_response(),

        _ => get_error("Error, this nft_type is not correct".into()),
    }
}
