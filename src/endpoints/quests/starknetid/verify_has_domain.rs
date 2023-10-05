use std::sync::Arc;

use crate::{
    models::AppState,
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

#[derive(Deserialize)]
pub struct StarknetIdQuery {
    addr: FieldElement,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StarknetIdQuery>,
) -> impl IntoResponse {
    let task_id = 1;
    let addr = &query.addr;

    // get starkname from address
    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.starknetid_contracts.naming_contract,
                entry_point_selector: selector!("address_to_domain"),
                calldata: vec![*addr],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    match call_result {
        Ok(result) => {
            let domain_len = i64::from_str_radix(&FieldElement::to_string(&result[0]), 16).unwrap();

            if domain_len > 0 {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("You don't own a stark domain".to_string())
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
