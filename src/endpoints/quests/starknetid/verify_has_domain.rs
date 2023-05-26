use std::sync::Arc;

use crate::{
    models::{AppState, CompletedTasks},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, options::UpdateOptions};
use serde::Deserialize;
use serde_json::json;
use starknet::{
    core::types::{BlockId, CallFunction, FieldElement},
    macros::selector,
    providers::{Provider, SequencerGatewayProvider},
};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct StarknetIdQuery {
    addr: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StarknetIdQuery>,
) -> impl IntoResponse {
    let task_id = 1;
    let addr = &query.addr;

    // get starkname from address
    let provider = SequencerGatewayProvider::starknet_alpha_mainnet();
    let call_result = provider
        .call_contract(
            CallFunction {
                contract_address: FieldElement::from_str(
                    &state.conf.starknetid_contracts.naming_contract,
                )
                .unwrap(),
                entry_point_selector: selector!("address_to_domain"),
                calldata: vec![FieldElement::from_str(addr).unwrap()],
            },
            BlockId::Latest,
        )
        .await;

    match call_result {
        Ok(result) => {
            let domain_len =
                i64::from_str_radix(&FieldElement::to_string(&result.result[0]), 16).unwrap();

            if domain_len > 0 {
                let completed_tasks_collection =
                    state.db.collection::<CompletedTasks>("completed_tasks");
                let filter = doc! { "address": addr, "task_id": task_id };
                let update = doc! { "$setOnInsert": { "address": addr, "task_id": task_id } };
                let options = UpdateOptions::builder().upsert(true).build();

                let result = completed_tasks_collection
                    .update_one(filter, update, options)
                    .await;

                match result {
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
