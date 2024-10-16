use std::sync::Arc;

use crate::{
    models::{AppState, QuestTaskDocument, VerifyQuery},
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
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

#[route(get, "/quests/proscore/verify_signers")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = query.task_id.unwrap();
    // Check the task verify_endpoint is quests/proscore/verify_signers
    let tasks_collection = state.db.collection("tasks");
    let task: QuestTaskDocument = tasks_collection
        .find_one(doc! {"id": &task_id}, None)
        .await
        .unwrap()
        .unwrap();
    if task.verify_endpoint != "quests/proscore/verify_signers" {
        return get_error("Invalid task".to_string());
    }
    match state
        .provider
        .call(
            FunctionCall {
                contract_address: query.addr,
                entry_point_selector: selector!("get_signers"),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
    {
        Ok(result) => match parse_braavos_signers(&result) {
            Ok(true) => match state.upsert_completed_task(query.addr, task_id).await {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            },
            Ok(false) => get_error("You have not enabled 2FA in your wallet".to_string()),
            Err(e) => get_error(format!("Error while parsing Braavos signers: {}", e)),
        },
        Err(_) => get_error("You must use a Braavos wallet to complete this task".to_string()),
    }
}

fn parse_braavos_signers(data: &[FieldElement]) -> Result<bool, String> {
    if data.len() < 4 {
        return Err("Input data is too short to parse".to_string());
    }
    let result_len = FieldElement::from(data.len());

    // Determine how many values are associated with stark_signers
    let stark_count = data[0];

    // Calculate the start index for secp256r1 based on stark_signers count
    let secp_start_idx = FieldElement::ONE + stark_count;
    if secp_start_idx >= result_len {
        return Err("Data array does not contain secp256r1 info".to_string());
    }
    let secp_start_idx_dec = FieldElement::to_string(&secp_start_idx)
        .parse::<usize>()
        .unwrap();
    let secp_count = data[secp_start_idx_dec];
    if secp_count > FieldElement::ZERO {
        return Ok(true);
    }
    if secp_start_idx + secp_count >= result_len {
        return Err("Data array does not contain enough values for secp256r1".to_string());
    }

    // Calculate the start index for webauthn based on previous values
    let webauthn_start_idx = secp_start_idx + FieldElement::ONE + secp_count;
    let webauthn_start_idx_usize = FieldElement::to_string(&webauthn_start_idx)
        .parse::<usize>()
        .unwrap();
    let webauthn_count = data[webauthn_start_idx_usize];
    if webauthn_count > FieldElement::ZERO {
        return Ok(true);
    }

    Ok(false)
}
