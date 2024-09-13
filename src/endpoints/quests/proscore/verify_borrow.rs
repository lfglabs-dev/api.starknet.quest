use std::{str::FromStr, sync::Arc};

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

#[route(get, "/quests/proscore/verify_borrow")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 190;
    match state
        .provider
        .call(
            FunctionCall {
                contract_address: FieldElement::from_str(
                    "0x04c0a5193d58f74fbace4b74dcf65481e734ed1714121bdc571da345540efa05",
                )
                .unwrap(),
                entry_point_selector: selector!("get_user_debt_for_token"),
                calldata: vec![
                    query.addr,
                    FieldElement::from_str(
                        "0x075afe6402ad5a5c20dd25e10ec3b3986acaa647b77e4ae24b0cbc9a54a27a87",
                    )
                    .unwrap(),
                ],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
    {
        Ok(result) => match parse_res(&result) {
            Ok(true) => match state.upsert_completed_task(query.addr, task_id).await {
                Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                Err(e) => get_error(format!("{}", e)),
            },
            Ok(false) => get_error("You must borrow 10 EKUBO tokens".to_string()),
            Err(e) => get_error(format!("Error while parsing Braavos signers: {}", e)),
        },
        Err(e) => get_error(format!("Error while verifying borrow: {}", e)),
    }
}

fn parse_res(data: &[FieldElement]) -> Result<bool, String> {
    let min: FieldElement = FieldElement::from_str("9000000000000000000").unwrap();
    let value = data[0];
    Ok(value >= min)
}
