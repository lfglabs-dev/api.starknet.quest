use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::{Bson, doc,from_document,DateTime};
use serde::Deserialize;
use std::sync::Arc;
use chrono::Utc;
#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

#[route(get, "/get_quest", crate::endpoints::get_quest)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    let current_time = chrono::Utc::now().timestamp_millis();
    let pipeline = [
        doc! {
            "$match": doc! {
                "disabled": false,
                "start_time": doc! {
                    "$lte": current_time
                }
            }
        },
        doc! {
            "$addFields": doc! {
                "expired": doc! {
                    "$cond": [
                        doc! {
                            "$and": [
                                doc! {
                                    "$gte": [
                                        "$expiry",
                                        0
                                    ]
                                },
                                doc! {
                                    "$lt": [
                                        "$expiry",
                                        "$$NOW"
                                    ]
                                }
                            ]
                        },
                        true,
                        false
                    ]
                },
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(mut quest) = from_document::<QuestDocument>(document) {
                            if let Some(expiry) = &quest.expiry {
                                quest.expiry_timestamp = Some(expiry.to_string());
                            }
                            return (StatusCode::OK, Json(quest)).into_response();
                        }
                    }
                    _ => continue,
                }
            }
            get_error("Quest not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
