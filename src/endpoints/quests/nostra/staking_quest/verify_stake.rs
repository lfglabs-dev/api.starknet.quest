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

#[route(
get,
"/quests/nostra/staking_quest/verify_stake",
crate::endpoints::quests::nostra::staking_quest::verify_stake
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 133;
    let addr = &query.addr;
    let balance_calldata = vec![*addr];
    let balance_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.quests.nostra.staking_contract,
                entry_point_selector: selector!("balance_of"),
                calldata: balance_calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    let user_balance = match balance_result {
        Ok(result) => result[0],
        Err(e) => return get_error(format!("{}", e)),
    };

    if user_balance == FieldElement::ZERO {
        return get_error("You didn't stake any STRK.".to_string());
    }

    let calldata = vec![user_balance];
    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.quests.nostra.staking_contract,
                entry_point_selector: selector!("convert_to_assets"),
                calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    match call_result {
        Ok(result) => {
            if result[0] < FieldElement::from_dec_str("10").unwrap() {
                get_error("You need to stake atleast 10 STRK".to_string())
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
