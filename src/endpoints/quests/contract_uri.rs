use crate::models::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_auto_routes::route;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct TokenURI {
    name: String,
    description: String,
    image: String,
    banner_image_url: String,
    external_link: String,
}

#[route(get, "/quests/contract_uri", crate::endpoints::quests::contract_uri)]
pub async fn handler(State(state): State<Arc<AppState>>) -> Response {
    let response = TokenURI {
        name: "Starknet Quest".into(),
        description: "Starknet Quest - The Collection of your Starknet achievements".into(),
        image: format!(
            "{}/visuals/starknetquest.webp",
            state.conf.variables.app_link
        ),
        banner_image_url: format!(
            "{}/visuals/starknetquestBanner.webp",
            state.conf.variables.app_link
        ),
        external_link: format!("{}/", state.conf.variables.app_link),
    };
    (StatusCode::OK, Json(response)).into_response()
}
