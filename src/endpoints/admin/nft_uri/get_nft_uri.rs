use crate::middleware::auth::auth_middleware;
use crate::models::NFTUri;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: i64,
}

#[route(get, "/admin/nft_uri/get_nft_uri", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<NFTUri>("nft_uri");
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "quest_id": query.id
            }
        },
        doc! {
            "$project": doc! {
            "_id": 0
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        return (StatusCode::OK, Json(document)).into_response();
                    }
                    _ => continue,
                }
            }
            get_error("NFT Uri not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
