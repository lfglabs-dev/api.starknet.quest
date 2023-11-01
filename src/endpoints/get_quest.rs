use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use mongodb::bson;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    let current_timestamp = bson::DateTime::from_millis(Utc::now().timestamp_millis());
    let filter = doc! {
        "$and": [
            {
                "id": query.id
            },
            {
                "$or": [
                    {
                        "expiry": {
                            "$exists": false
                        }
                    },
                    {
                        "expiry": {
                            "$gt": current_timestamp
                        }
                    }
                ]
            },
            {
                "disabled": false
            }
        ]
    };
    match collection.find_one(filter, None).await {
        Ok(Some(mut quest)) => {
            if let Some(expiry) = &quest.expiry {
                let timestamp = expiry.timestamp_millis().to_string();
                quest.expiry_timestamp = Some(timestamp);
                let current_timestamp = bson::DateTime::from_millis(Utc::now().timestamp_millis());
                quest.expired = Some(expiry < &current_timestamp);
            }
            (StatusCode::OK, Json(quest)).into_response()
        }
        Ok(None) => get_error("Quest not found".to_string()),
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
