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
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

#[route(get, "/boost/get_boost", crate::endpoints::quest_boost::get_boost)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("boosts");
    let pipeline = [
        doc! {
            "$match": {
                "id": query.id
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "boost_claims",
                "localField": "id",
                "foreignField": "id",
                "as": "claim_detail"
            }
        },
        doc! {
        "$addFields": doc! {
            "claimed": doc! {
                "$anyElementTrue": doc! {
                    "$map": doc! {
                        "input": "$claim_detail",
                        "as": "claimDetail",
                        "in": doc! {
                        "$eq": [
                            doc! {
                                "$ifNull": [
                                    "$$claimDetail._cursor.to",
                                    null
                                ]
                            },
                            null
                        ]
                        }
                    }
                }
            }
        },
        },
        doc! {
            "$unset": "claim_detail"
        },
        doc! {
            "$project":{
            "_id":0
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
            get_error("Quest not found".to_string())
        }
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
