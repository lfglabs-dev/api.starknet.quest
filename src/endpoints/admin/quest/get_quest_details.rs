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
use mongodb::bson::{doc,};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

#[route(
get,
"/admin/get_quest_details",
crate::endpoints::admin::quest::get_quest_details
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
    let pipeline = vec![
        doc! {
        "$match": doc! {
            "id": query.id
        }
    },
        doc! {
        "$lookup": doc! {
            "from": "boosts",
            "let": doc! {
                "localFieldValue": "$id"
            },
            "pipeline": [
                doc! {
                    "$match": doc! {
                        "$expr": doc! {
                            "$and": [
                                doc! {
                                    "$in": [
                                        "$$localFieldValue",
                                        "$quests"
                                    ]
                                }
                            ]
                        }
                    }
                },
                doc! {
                    "$project": doc! {
                        "_id": 0,
                        "hidden": 0
                    }
                }
            ],
            "as": "boosts"
        }
    }
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
            get_error("Quest not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
