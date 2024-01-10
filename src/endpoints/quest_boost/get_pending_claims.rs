use crate::utils::to_hex;
use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let address = to_hex(query.addr);
    let collection = state.db.collection::<QuestDocument>("boosts");
    let pipeline = [
        doc! {
            "$match": {
                "winner":address
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "boost_claims",
                "let": doc! {
                    "localId": "$id",
                    "localWinner": "$winner"
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "$expr": doc! {
                                "$and": [
                                    doc! {
                                        "$eq": [
                                            "$id",
                                            "$$localId"
                                        ]
                                    },
                                    doc! {
                                        "$winner": { "$in": ["$$localWinner"] }
                                    },
                                    doc! {
                                        "$eq": [
                                            "$_cursor.to",
                                            null
                                        ]
                                    }
                                ]
                            }
                        }
                    }
                ],
                "as": "boost_claims"
            }
        },
        doc! {
            "$match": doc! {
                "$expr": doc! {
                    "$eq": [
                        doc! {
                            "$size": "$boost_claims"
                        },
                        0
                    ]
                }
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "boost_claims": 0,
                "hidden":0
            }
        },
    ];

    match collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut res = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        res.push(document);
                    }
                    _ => continue,
                }
            }
            return (StatusCode::OK, Json(res)).into_response();
        }
        Err(e) => {
            println!("Error querying claims: {}", e);
            get_error("Error querying claims".to_string())
        }
    }
}
