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
    macros::selector,
    providers::Provider,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VerifyBalanceQuery {
    pub addr: FieldElement,
    pub task_id: u32,
}

#[route(get, "/quests/verify_balance")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyBalanceQuery>,
) -> impl IntoResponse {
    let task_id = query.task_id;
    // Get task in db
    let task_collection = state.db.collection("tasks");
    let task: QuestTaskDocument = task_collection
        .find_one(doc! {"id": task_id}, None)
        .await
        .unwrap()
        .unwrap();

    if task.task_type != Some("balance".to_string()) {
        return get_error("Invalid task type.".to_string());
    }

    let addr = &query.addr;
    let utils_contract = state.conf.quests.utils_contract;

    let mut calldata = vec![addr.clone(), task.contracts.clone().unwrap().len().into()];
    calldata.append(&mut task.contracts.unwrap().clone());

    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: utils_contract,
                entry_point_selector: selector!("sum_balances"),
                calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    match call_result {
        Ok(result) => {
            if result[0] < FieldElement::from_dec_str("3000000000000000").unwrap() {
                get_error("You didn't invest (enough).".to_string())
            } else {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
