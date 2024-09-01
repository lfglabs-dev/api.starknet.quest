use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, Extension},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    id: i64,
    quest_id: i64,
    name: String,
    href: String,
    cta: String,
    verify_endpoint: String,
    verify_endpoint_type: String,
    verify_redirect: Option<String>,
    desc: String,
    quiz_name: Option<i64>,
    task_type: Option<String>,
    discord_guild_id: Option<String>,
}

#[derive(Deserialize)]
pub struct GetTasksQuery {
    quest_id: u32,
}

pub async fn handler(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<GetTasksQuery>,
) -> impl IntoResponse {
    let pipeline = vec![
        doc! { "$match": { "quest_id": query.quest_id } },
        doc! {
            "$lookup": {
                "from": "quests",
                "localField": "quest_id",
                "foreignField": "id",
                "as": "quest"
            }
        },
        doc! { "$unwind": "$quest" },
        doc! {
            "$addFields": {
                "sort_order": doc! {
                    "$switch": {
                        "branches": [
                            {
                                "case": doc! { "$eq": ["$verify_endpoint_type", "quiz"] },
                                "then": 1
                            },
                            {
                                "case": doc! { "$eq": ["$verify_endpoint_type", "default"] },
                                "then": 2
                            }
                        ],
                        "default": 3
                    }
                }
            }
        },
        doc! { "$sort": { "sort_order": 1 } },
        doc! {
            "$project": {
                "_id": 0,
                "id": 1,
                "quest_id": 1,
                "name": 1,
                "href": 1,
                "cta": 1,
                "verify_endpoint": 1,
                "verify_redirect" : 1,
                "verify_endpoint_type": 1,
                "desc": 1,
                "quiz_name": 1,
                "task_type": 1,
                "discord_guild_id": 1,
            }
        },
    ];

    let tasks_collection = state.db.collection::<UserTask>("tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut tasks: Vec<UserTask> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(task) = from_document::<UserTask>(document) {
                            tasks.push(task);
                        }
                    }
                    _ => continue,
                }
            }
            if tasks.is_empty() {
                get_error("No tasks found for this quest_id".to_string())
            } else {
                (StatusCode::OK, Json(tasks)).into_response()
            }
        }
        Err(_) => get_error("Error querying tasks".to_string()),
    }
}

pub fn get_tasks_routes() -> Router {
    Router::new().route("/get_tasks", get(handler))
}
