use crate::{
    models::{AppState, QuestDocument,JWTClaims},
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
use axum::http::HeaderMap;
use jsonwebtoken::{Validation,Algorithm,decode,DecodingKey};


#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: i32,
}

#[route(
    get,
    "/admin/quest/get_quest",
    crate::endpoints::admin::quest::get_quest
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref())  as String;
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
        pipeline.insert(1, doc! {
            "$match": doc! {
                "issuer": user,
            }
        });
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
