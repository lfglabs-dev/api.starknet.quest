use std::sync::Arc;

use crate::models::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, options::UpdateOptions};
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::{
    core::types::{BlockId, CallFunction, FieldElement},
    macros::{felt, selector, short_string},
    providers::{Provider, SequencerGatewayProvider},
};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct StarknetIdQuery {
    addr: String,
}

#[derive(Deserialize)]
pub struct CompletedTasks {
    address: String,
    task_id: u32,
}

#[derive(Serialize)]
pub struct QueryError {
    pub error: String,
    pub res: bool,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StarknetIdQuery>,
) -> impl IntoResponse {
    let task_id = 6;
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
            // get starknet id from domain
            let id_call_result = provider
                .call_contract(
                    CallFunction {
                        contract_address: FieldElement::from_str(
                            &state.conf.starknetid_contracts.naming_contract,
                        )
                        .unwrap(),
                        entry_point_selector: selector!("domain_to_token_id"),
                        calldata: result.result,
                    },
                    BlockId::Latest,
                )
                .await;

            match id_call_result {
                Ok(starknet_id) => {
                    // get verifier data from starknet id
                    let verifier_call_result = provider
                        .call_contract(
                            CallFunction {
                                contract_address: FieldElement::from_str(
                                    &state.conf.starknetid_contracts.identity_contract,
                                )
                                .unwrap(),
                                entry_point_selector: selector!("get_verifier_data"),
                                calldata: vec![
                                    starknet_id.result[0],
                                    short_string!("twitter"),
                                    FieldElement::from_str(
                                        &state.conf.starknetid_contracts.verifier_contract,
                                    )
                                    .unwrap(),
                                ],
                            },
                            BlockId::Latest,
                        )
                        .await;

                    match verifier_call_result {
                        Ok(verifier_data) => {
                            if verifier_data.result[0] != felt!("0") {
                                let completed_tasks_collection =
                                    state.db.collection::<CompletedTasks>("completed_tasks");
                                let filter = doc! { "address": addr, "task_id": task_id };
                                let update = doc! { "$setOnInsert": { "address": addr, "task_id": task_id } };
                                let options = UpdateOptions::builder().upsert(true).build();

                                let result = completed_tasks_collection
                                    .update_one(filter, update, options)
                                    .await;

                                match result {
                                    Ok(_) => {
                                        (StatusCode::OK, Json(json!({"res": true}))).into_response()
                                    }
                                    Err(e) => {
                                        let error = QueryError {
                                            error: format!("{}", e),
                                            res: false,
                                        };
                                        (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
                                            .into_response()
                                    }
                                }
                            } else {
                                let error = QueryError {
                                    error: String::from(
                                        "You have not verified your Twitter account",
                                    ),
                                    res: false,
                                };
                                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                            }
                        }
                        Err(e) => {
                            println!("error: {}", e);
                            let error = QueryError {
                                error: format!("{}", e),
                                res: false,
                            };
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                        }
                    }
                }
                Err(e) => {
                    let error = QueryError {
                        error: format!("{}", e),
                        res: false,
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            let error = QueryError {
                error: format!("{}", e),
                res: false,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
