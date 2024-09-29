use std::sync::Arc;

use crate::{
    models::{AppState, QuestTaskDocument},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    providers::Provider,
};
use regex::Regex;
use crate::utils::parse_string;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VerifyContractQuery {
    pub addr: FieldElement,
    pub task_id: u32,
}

#[route(get, "/quests/verify_contract")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyContractQuery>,
) -> impl IntoResponse {
    let task_id = query.task_id;
    // Get task from db
    let task_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let task = match task_collection.find_one(doc! {"id": task_id}, None).await {
        Ok(Some(task)) => task,
        Ok(None) => return get_error("Task not found".to_string()),
        Err(e) => return get_error(format!("Database error: {}", e)),
    };
    if task.task_type != Some("contract".to_string()) {
        return get_error("Invalid task type.".to_string());
    }

    let addr = &query.addr;
    if let Some(calls) = task.calls {
        for call in calls {

            let contract_address = match FieldElement::from_hex_be(&call.contract) {
                Ok(address) => address,
                Err(e) => return get_error(format!("Invalid contract address: {}", e)),
            };


            let calldata: Vec<FieldElement> = match call.call_data
                .iter()
                .map(|s| {
        
                    let replaced_calldata = parse_string(s, FieldElement::from_hex_be(s).unwrap());
                    FieldElement::from_hex_be(&replaced_calldata)
                })
                .collect::<Result<Vec<FieldElement>, _>>()
            {
                Ok(data) => data,
                Err(e) => return get_error(format!("Invalid calldata: {}", e)),
            };


            let entry_point_selector = match FieldElement::from_hex_be(&call.entry_point) {
                Ok(selector) => selector,
                Err(e) => return get_error(format!("Invalid entry point: {}", e)),
            };


            let call_result = state
                .provider
                .call(
                    FunctionCall {
                        contract_address,
                        entry_point_selector,
                        calldata,
                    },
                    BlockId::Tag(BlockTag::Latest),
                )
                .await;


            match call_result {
                Ok(result) => {
                    let regex = match Regex::new(&call.regex) {
                        Ok(re) => re,
                        Err(e) => return get_error(format!("Invalid regex: {}", e)),
                    };
                    let result_str = result.iter().map(|&r| r.to_string()).collect::<Vec<String>>().join(",");

        
                    if !regex.is_match(&result_str) {
                        return get_error("Contract call result does not match the expected pattern.".to_string());
                    }
                }
                Err(e) => return get_error(format!("Contract call failed: {}", e)),
            }
        }

        // All calls succeeded and matched their regexes
        match state.upsert_completed_task(*addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(e) => get_error(format!("Failed to update completed task: {}", e)),
        }
    } else {
        get_error("No calls specified for this task.".to_string())
    }
}
