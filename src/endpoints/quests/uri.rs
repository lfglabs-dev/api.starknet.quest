use crate::models::{AppState, NFTUri};
use crate::utils::get_error;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_auto_routes::route;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::StreamExt;
use mongodb::bson::{doc, from_document};

#[derive(Serialize)]
pub struct TokenURI {
    name: String,
    description: String,
    image: String,
    attributes: Option<Vec<Attribute>>,
}

#[derive(Serialize,Deserialize)]
pub struct Attribute {
    trait_type: String,
    value: u32,
}

#[derive(Deserialize)]
pub struct LevelQuery {
    level: Option<String>,
}

#[route(get, "/quests/uri", crate::endpoints::quests::uri)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(level_query): Query<LevelQuery>,
) -> Response {
    let level = level_query
        .level
        .and_then(|level_str| level_str.parse::<i64>().ok());

    let uri_collection = state.db.collection::<NFTUri>("nft_uri");
    let pipeline = vec![
        doc! {
            "$match":{
                "id":&level.unwrap()
            }
        }
    ];

    match uri_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                return match result {
                    Ok(document) => {
                        if let Ok(mut nft_uri) = from_document::<NFTUri>(document) {
                            return (StatusCode::OK,
                             Json(TokenURI {
                                 name: (&*nft_uri.name).to_string(),
                                 description: (&*nft_uri.description).to_string(),
                                 image: format!("{}{}", state.conf.variables.app_link, &*nft_uri.image),
                                 attributes: None,
                             })).into_response()
                        }
                        get_error("Error querying NFT URI".to_string())
                    }
                    _ => get_error("Error querying NFT URI".to_string()),
                };
            }
            get_error("NFT URI not found".to_string())
        }
        Err(_) => get_error("Error querying NFT URI".to_string()),
    }
}
