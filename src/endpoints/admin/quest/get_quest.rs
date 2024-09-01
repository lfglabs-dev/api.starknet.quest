use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, Extension},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;
use futures::StreamExt;

use crate::models::JWTClaims;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::Algorithm;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: i32,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Replace this with actual implementation of `check_authorization!` macro
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestDocument>("quests");
    
    let mut pipeline = vec![
        doc! {
            "$match": doc! {
                "id": query.id,
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
                        }
                    }
                ],
                "as": "boosts"
            },
        },
        doc! {
            "$project": doc! {
                "_id": 0
            }
        },
    ];

    if user != "super_user" {
        pipeline.insert(
            1,
            doc! {
                "$match": doc! {
                    "issuer": user,
                }
            },
        );
    }

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

pub fn get_quest_routes() -> Router {
    Router::new().route("/get_quest", get(handler))
}
