use std::sync::Arc;

use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

#[route(get, "/quests/braavos/myswap/verify_added_liquidity")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 76;
    let addr = &query.addr;

    // check if user has provider liquidity
    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.quests.myswap.contract,
                entry_point_selector: selector!("balance_of"),
                calldata: vec![*addr],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    match call_result {
        Ok(result) => {
            if result[0] == FieldElement::ZERO && result[1] == FieldElement::ZERO {
                get_error("You haven't provided any liquidity on MySwap.".to_string())
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
