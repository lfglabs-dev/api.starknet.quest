// src/endpoints/admin/quest/get_quest_users_csv.rs
use crate::middleware::auth::auth_middleware;
use crate::models::{AppState, CompletedTaskDocument, QuestDocument, QuestTaskDocument};
use crate::utils::{get_error, to_hex, verify_quest_auth}; 
use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use axum_auto_routes::route;
use csv::WriterBuilder; // import from csv crate
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use starknet::core::types::FieldElement;
use std::collections::HashSet;
use std::sync::Arc;


pub_struct!(Deserialize; GetQuestUsersParams {
    quest_id: i64,
});

#[route(get, "/admin/quests/get_quest_users_csv", auth_middleware)]
pub async fn get_quest_users_csv_handler(
    State(state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>, 
    Query(params): Query<GetQuestUsersParams>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let completed_tasks_collection = state
        .db
        .collection::<CompletedTaskDocument>("completed_tasks");
    let quests_collection = state.db.collection::<QuestDocument>("quests");

    // Verify quest authorization
    let res = verify_quest_auth(_sub, &quests_collection, &params.quest_id).await;
    if !res {
        return get_error("User is not authorized to view this quest's users.".to_string());
    }

    // Fetch all task IDs for the given quest_id
    let task_filter = doc! { "quest_id": params.quest_id };
    let task_cursor = match tasks_collection.find(task_filter, None).await {
        Ok(cursor) => cursor,
        Err(e) => return get_error(format!("Error fetching tasks: {}", e)),
    };

    let task_ids_result: Result<Vec<u32>, _> =
        task_cursor.map_ok(|doc| doc.id as u32).try_collect().await;

    let task_ids = match task_ids_result {
        Ok(ids) => ids,
        Err(e) => return get_error(format!("Error processing tasks: {}", e)),
    };

    if task_ids.is_empty() {
        return get_error(format!("No tasks found for quest_id {}", params.quest_id));
    }

    // Fetch all completed tasks for these task IDs
    let completed_task_filter = doc! { "task_id": { "$in": &task_ids } };
    let completed_task_cursor = match completed_tasks_collection
        .find(completed_task_filter, None)
        .await
    {
        Ok(cursor) => cursor,
        Err(e) => return get_error(format!("Error fetching completed tasks: {}", e)),
    };

    let completed_tasks_result: Result<Vec<CompletedTaskDocument>, _> =
        completed_task_cursor.try_collect().await;

    let users: Vec<String> = match completed_tasks_result {
        Ok(completed_tasks) => {
            let user_set: HashSet<String> = completed_tasks
                .into_iter()
                .map(|task| task.address().to_string()) // address() returns FieldElement, .to_string() gives decimal
                .collect();

            user_set
                .into_iter()
                .filter_map(|addr_dec_str| FieldElement::from_dec_str(&addr_dec_str).ok().map(to_hex))
                .collect()
        }
        Err(e) => return get_error(format!("Error processing completed tasks: {}", e)),
    };

    // Convert data to CSV
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);

    if let Err(e) = wtr.write_record(&["user_address"]) {
        eprintln!("Error writing CSV header: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )],
            "Error generating CSV data".to_string(),
        )
            .into_response();
    }

    for user_address in users {
        if let Err(e) = wtr.write_record(&[&user_address]) {
            eprintln!("Error writing CSV record: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                "Error generating CSV data".to_string(),
            )
                .into_response();
        }
    }

    let csv_data = match wtr.into_inner() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error finalizing CSV data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                "Error generating CSV data".to_string(),
            )
                .into_response();
        }
    };

    let csv_string = match String::from_utf8(csv_data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error converting CSV data to UTF-8 string: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                "Error generating CSV data".to_string(),
            )
                .into_response();
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    let filename = format!("quest_users_{}.csv", params.quest_id);
    match HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)) {
        Ok(val) => {
            headers.insert(header::CONTENT_DISPOSITION, val);
        }
        Err(e) => {
            eprintln!("Error creating Content-Disposition header value: {}", e);
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"quest_users.csv\""),
            );
        }
    }

    (StatusCode::OK, headers, csv_string).into_response()
}