use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    models::AppState,
    utils::{get_error, CompletedTasksTrait},
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

pub async fn execute_has_root_domain(
    state: Arc<AppState>,
    addr: &FieldElement,
    task_id: u32,
) -> impl IntoResponse {
    // get starkname from address
    let call_result = state
        .provider
        .call(
            FunctionCall {
                contract_address: state.conf.starknetid_contracts.naming_contract,
                entry_point_selector: selector!("address_to_domain"),
                calldata: vec![*addr, FieldElement::ZERO],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    match call_result {
        Ok(result) => {
            let domain_len = i64::from_str_radix(&FieldElement::to_string(&result[0]), 16).unwrap();

            if domain_len == 1 {
                // get expiry
                let Ok(expiry_result) = state
                    .provider
                    .call(
                        FunctionCall {
                            contract_address: state.conf.starknetid_contracts.naming_contract,
                            entry_point_selector: selector!("domain_to_expiry"),
                            calldata: vec![ FieldElement::ONE, result[1] ],
                        },
                        BlockId::Tag(BlockTag::Latest),
                    )
                    .await else {
                        return get_error("error querying expiry".to_string())
                    };
                let Ok(expiry) : Result<u64, _> = expiry_result[0].try_into() else {
                        return get_error("error reading expiry".to_string())
                    };
                let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => return get_error("system time before UNIX EPOCH".to_string()),
                };
                if expiry < now {
                    return get_error("expired domain".to_string());
                }

                match state.upsert_completed_task(*addr, task_id).await {
                    Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                }
            } else {
                get_error("Invalid domain: subdomains are not eligible".to_string())
            }
        }
        Err(e) => get_error(format!("{}", e)),
    }
}
