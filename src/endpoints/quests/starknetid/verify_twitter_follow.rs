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
    core::types::{BlockId, CallContractResult, CallFunction, FieldElement},
    macros::{felt, selector, short_string},
    providers::Provider,
};
use std::sync::Arc;

async fn call_contract_helper(
    state: &AppState,
    contract: FieldElement,
    entry_point: FieldElement,
    calldata: Vec<FieldElement>,
) -> Result<CallContractResult, String> {
    let result = state
        .provider
        .call_contract(
            CallFunction {
                contract_address: contract,
                entry_point_selector: entry_point,
                calldata,
            },
            BlockId::Latest,
        )
        .await;

    result.map_err(|e| format!("{}", e))
}

async fn check_if_user_follows_starknet_quest(_twitter_id: &str) -> bool {
    true
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    async fn inner(
        state: Arc<AppState>,
        query: VerifyQuery,
    ) -> Result<(StatusCode, Json<serde_json::Value>), String> {
        let task_id = 7;
        let addr = &query.addr;

        let domain_res = call_contract_helper(
            &state,
            state.conf.starknetid_contracts.naming_contract,
            selector!("address_to_domain"),
            vec![*addr],
        )
        .await?;

        let id_res = call_contract_helper(
            &state,
            state.conf.starknetid_contracts.naming_contract,
            selector!("domain_to_token_id"),
            domain_res.result,
        )
        .await?;

        let twitter_verifier_data = call_contract_helper(
            &state,
            state.conf.starknetid_contracts.identity_contract,
            selector!("get_verifier_data"),
            vec![
                id_res.result[0],
                short_string!("twitter"),
                state.conf.starknetid_contracts.verifier_contract,
            ],
        )
        .await?;

        let Some(twitter_felt) =  twitter_verifier_data.result.first() else {
           return Err("Unable to read twitter id".to_string())
        };
        let follows_starknet_quest =
            check_if_user_follows_starknet_quest(&twitter_felt.to_string()).await;

        if twitter_verifier_data.result[0] != felt!("0") && follows_starknet_quest {
            match state.upsert_completed_task(query.addr, task_id).await {
                Ok(_) => Ok((StatusCode::OK, Json(json!({"res": true})))),
                Err(e) => Err(e.to_string()),
            }
        } else if twitter_verifier_data.result[0] == felt!("0") {
            Err("You have not verified your Twitter account".to_string())
        } else if !follows_starknet_quest {
            Err("You are not following @starknet_quest on Twitter".to_string())
        } else {
            Err("Unknown error".to_string())
        }
    }

    match inner(state, query).await {
        Ok(val) => val.into_response(),
        Err(err) => get_error(err),
    }
}
