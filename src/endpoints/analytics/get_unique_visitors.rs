use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

#[route(
get,
"/analytics/get_unique_visitors",
crate::endpoints::analytics::get_unique_visitors
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let quest_id = query.id;
    let page_id = "quest_".to_owned() + quest_id.to_string().as_str();
    let total_viewers_pipeline = vec![
        doc! {
            "$match": doc! {
                "viewed_page_id": page_id
            }
        },
        doc! {
            "$count":"total_viewers"
        },
    ];

    match state
        .db
        .collection::<QuestTaskDocument>("unique_viewers")
        .aggregate(total_viewers_pipeline, None)
        .await
    {
        Ok(mut cursor) => {
            let mut result = 0;
            return match cursor.try_next().await {
                Ok(Some(doc)) => {
                    result = doc.get("total_viewers").unwrap().as_i32().unwrap();
                    (StatusCode::OK, Json(result)).into_response()
                }
                Ok(None) => (StatusCode::OK, Json(result)).into_response(),
                Err(_) => get_error("Error querying quest".to_string()),
            };
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
