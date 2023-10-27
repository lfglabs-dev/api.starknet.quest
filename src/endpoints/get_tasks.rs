use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document, Document};
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    id: u32,
    quest_id: u32,
    name: String,
    href: String,
    cta: String,
    verify_endpoint: String,
    verify_endpoint_type: String,
    verify_redirect: Option<String>,
    desc: String,
    completed: bool,
    quiz_name: Option<String>,
}

#[derive(Deserialize)]
pub struct GetTasksQuery {
    quest_id: u32,
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetTasksQuery>,
) -> impl IntoResponse {
    let pipeline = vec![
        doc! { "$match": { "quest_id": query.quest_id } },
        doc! {
            "$lookup": {
                "from": "completed_tasks",
                "let": { "task_id": "$id" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": { "$eq": [ "$task_id", "$$task_id" ] },
                            "address": query.addr.to_string(),
                        },
                    },
                ],
                "as": "completed",
            }
        },
        doc! {
            "$lookup": {
                "from": "quests",
                "localField": "quest_id",
                "foreignField": "id",
                "as": "quest"
            }
        },
        doc! { "$unwind": "$quest" },
        doc! { "$match": { "quest.disabled": false } },
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
                "completed": { "$gt": [ { "$size": "$completed" }, 0 ] },
                "quiz_name": 1,
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut quiz_tasks: Vec<UserTask> = Vec::new();
            let mut social_medias_tasks: Vec<UserTask> = Vec::new();
            let mut default_tasks: Vec<UserTask> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(task) = from_document::<UserTask>(document) {
                            let endpoint_type = task.verify_endpoint_type.clone();
                            match endpoint_type.as_str() {
                                "quiz" => quiz_tasks.push(task),
                                "default" => default_tasks.push(task),
                                _ => social_medias_tasks.push(task),
                            }
                        }
                    }
                    _ => continue,
                }
            }
            quiz_tasks.sort_by(|a, b| a.id.cmp(&b.id));
            social_medias_tasks.sort_by(|a, b| a.id.cmp(&b.id));
            default_tasks.sort_by(|a, b| a.id.cmp(&b.id));
            let tasks: Vec<UserTask> = quiz_tasks
                .into_iter()
                .chain(default_tasks.into_iter())
                .chain(social_medias_tasks.into_iter())
                .collect();
            if tasks.is_empty() {
                get_error("No tasks found for this quest_id".to_string())
            } else {
                (StatusCode::OK, Json(tasks)).into_response()
            }
        }
        Err(_) => get_error("Error querying tasks".to_string()),
    }
}
