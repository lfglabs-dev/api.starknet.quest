use std::sync::Arc;

use crate::{
    models::{AppState, BuildingDocument, BuildingQuery},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document};

#[route(get, "/achievements/fetch_buildings")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BuildingQuery>,
) -> impl IntoResponse {
    let ids_str = &query.ids;
    let ids: Vec<u32> = ids_str.split(',').filter_map(|s| s.parse().ok()).collect();
    let buildings_collection = state.db.collection::<BuildingDocument>("buildings");
    let pipeline = vec![doc! {
        "$match": {
            "id": {
                "$in": ids
            }
        }
    }];

    match buildings_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut buildings: Vec<BuildingDocument> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(building) = from_document::<BuildingDocument>(document) {
                            buildings.push(building);
                        }
                    }
                    _ => continue,
                }
            }
            (StatusCode::OK, Json(buildings)).into_response()
        }
        Err(e) => get_error(format!("Error fetching user buildings: {}", e)),
    }
}
