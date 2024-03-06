use crate::models::{QuestTaskDocument};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}


#[route(get, "/analytics/get_quest_activity", crate::endpoints::analytics::get_quest_activity)]
pub async fn handler(State(state): State<Arc<AppState>>,
                     Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let quest_id = query.id;
    let day_wise_distribution = vec![
        doc! {
        "$match": doc! {
            "quest_id": quest_id
        }
    },
        doc! {
        "$group": doc! {
            "_id": null,
            "ids": doc! {
                "$push": "$id"
            }
        }
    },
        doc! {
        "$lookup": doc! {
            "from": "completed_tasks",
            "localField": "ids",
            "foreignField": "task_id",
            "as": "matching_documents"
        }
    },
        doc! {
        "$unwind": "$matching_documents"
    },
        doc! {
        "$replaceRoot": doc! {
            "newRoot": "$matching_documents"
        }
    },
        doc! {
        "$addFields": doc! {
            "createdDate": doc! {
                "$toDate": "$timestamp"
            }
        }
    },
        doc! {
        "$group": doc! {
            "_id": doc! {
                "$dateToString": doc! {
                    "format": "%Y-%m-%d %d",
                    "date": "$createdDate"
                }
            },
            "participants": doc! {
                "$sum": 1
            }
        }
    },
        doc! {
        "$sort": doc! {
            "_id": 1
        }
    },
    ];

    match state.db.collection::<QuestTaskDocument>("tasks").aggregate(day_wise_distribution, None).await {
        Ok(mut cursor) => {
            let mut day_wise_distribution = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        day_wise_distribution.push(document);
                    }
                    _ => continue,
                }
            }
            return (StatusCode::OK, Json(day_wise_distribution)).into_response();
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
