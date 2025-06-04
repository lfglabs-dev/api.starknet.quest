use axum::{
    body::Body,
    extract::{Extension, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_auto_routes::route;
use csv::Writer;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use starknet::core::types::FieldElement;
use std::sync::Arc;

use crate::{middleware::auth::auth_middleware, utils::to_hex};
use crate::{
    models::{AppState, CompletedTaskDocument, QuestTaskDocument},
    utils::get_error,
};

pub_struct!(Deserialize; GetQuestParticipantsParams {
    quest_id: i64,
});

#[route(get, "/admin/quests/get_quest_participants_csv", auth_middleware)]
pub async fn get_quest_participants_csv_handler(
    State(state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>,
    Query(params): Query<GetQuestParticipantsParams>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let completed_tasks_collection = state
        .db
        .collection::<CompletedTaskDocument>("completed_tasks");

    let task_filter = doc! { "quest_id": params.quest_id };
    let task_ids: Vec<i32> = match tasks_collection.find(task_filter, None).await {
        Ok(mut cursor) => {
            let mut ids = Vec::new();
            while let Some(doc) = cursor.next().await {
                match doc {
                    Ok(task) => ids.push(task.id),
                    Err(e) => return get_error(format!("Error processing tasks: {}", e)),
                }
            }
            ids
        }
        Err(e) => return get_error(format!("Error fetching tasks: {}", e)),
    };

    if task_ids.is_empty() {
        return get_error(format!("No tasks found for quest_id {}", params.quest_id));
    }

    let pipeline = vec![
        doc! { "$match": { "task_id": { "$in": &task_ids } } },
        doc! { "$group": {
            "_id": "$address",
            "task_ids": { "$addToSet": "$task_id" },
            "max_timestamp": { "$max": "$timestamp" }
        }},
        doc! { "$project": {
            "address": "$_id",
            "tasks_completed_count": { "$size": "$task_ids" },
            "quest_completion_timestamp": "$max_timestamp"
        }},
    ];

    let mut cursor = match completed_tasks_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => return get_error(format!("Error aggregating completed tasks: {}", e)),
    };

    let total_tasks = task_ids.len();

    let mut wtr = Writer::from_writer(vec![]);

    if let Err(e) = wtr.write_record(&["address", "quest_completion_timestamp"]) {
        return get_error(format!("Error writing CSV header: {}", e));
    }

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(doc) => {
                let address: String = match doc.get_str("address") {
                    Ok(addr) => to_hex(FieldElement::from_dec_str(addr).unwrap()),
                    Err(_) => continue,
                };

                let timestamp = match doc.get_i64("quest_completion_timestamp") {
                    Ok(ts) => ts,
                    Err(_) => continue,
                };

                let tasks_completed_count: usize = match doc.get_i32("tasks_completed_count") {
                    Ok(count) => count as usize,
                    Err(_) => continue,
                };

                if tasks_completed_count == total_tasks {
                    if let Err(e) = wtr.write_record(&[&address, &timestamp.to_string()]) {
                        return get_error(format!("Error writing CSV record: {}", e));
                    }
                }
            }
            Err(e) => return get_error(format!("Error processing aggregation results: {}", e)),
        }
    }

    match wtr.flush() {
        Ok(_) => {
            let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
            let mut response: axum::http::Response<Body> = Response::new(data.into());
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/csv"),
            );
            (StatusCode::OK, response).into_response()
        }
        Err(e) => get_error(format!("Error flushing CSV writer: {}", e)),
    }
}
