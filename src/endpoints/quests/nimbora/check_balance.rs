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
"/quests/nimbora/check_balance",
crate::endpoints::quests::nimbora::check_balance
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 144;
    let addr = &query.addr;
    let calldata = vec![*addr];
    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.quests.nimbora.contract,
                entry_point_selector: selector!("balance_of"),
                calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;


    match call_result {
        Ok(result) => {
            if result[0] < FieldElement::from_dec_str("4000000000000000").unwrap() {
                get_error("You didn't invest on nimbora.".to_string())
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
