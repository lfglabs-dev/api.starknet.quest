use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use crate::middleware::auth::auth_middleware;
use axum::{
    extract::{Query, State, Extension},
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
    id: i32,
}

#[route(get, "/admin/quest/get_quest", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
    Extension(sub): Extension<String>
) -> impl IntoResponse {
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

    if sub != "super_user" {
        pipeline.insert(
            1,
            doc! {
                "$match": doc! {
                    "issuer": sub,
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
