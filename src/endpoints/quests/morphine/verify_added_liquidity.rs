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
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

lazy_static::lazy_static! {
    static ref POOLS: Vec<FieldElement> = vec![
        FieldElement::from_hex_be(
            "0x78e348ee59bde05154a58ed149d289f0c80bc7c1dcd6aba6180f1845ea25ecc",
        ).unwrap(),
        FieldElement::from_hex_be(
            "0x012afd5a0a79ac3d1ee157221c15c54f541f73c145457505875557b0515dedaf",
        ).unwrap(),
        FieldElement::from_hex_be(
            "0x04f79B79aC514974c88A838Eb9fb180551A82D92fD01B2e02740bA3F1d382457",
        ).unwrap(),
        FieldElement::from_hex_be(
            "0x0170C38DcB19677c12d29C2Db96080E20DDc7D07e0Cf7fC0F3a076845b5c54e5",
        ).unwrap(),
    ];
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 41;
    let addr = &query.addr;

    // For each available pool we check if the user has provided liquidity
    let mut has_provided = false;
    for pool in POOLS.iter() {
        let call_result = state
            .provider
            .call(
                FunctionCall {
                    contract_address: *pool,
                    entry_point_selector: selector!("balanceOf"),
                    calldata: vec![*addr],
                },
                BlockId::Tag(BlockTag::Latest),
            )
            .await;

        match call_result {
            Ok(result) => {
                if result[0] != FieldElement::ZERO {
                    has_provided = true;
                    break;
                }
            }
            Err(e) => {
                return get_error(format!("{}", e));
            }
        }
    }

    if has_provided {
        match state.upsert_completed_task(query.addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(e) => get_error(format!("{}", e)),
        }
    } else {
        get_error("You didn't provide any liquidity on any pool.".to_string())
    }
}
