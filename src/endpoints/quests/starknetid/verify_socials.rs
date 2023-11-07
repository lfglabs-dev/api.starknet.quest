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
    macros::{felt, selector, short_string},
    providers::Provider,
};
use std::sync::Arc;

async fn call_contract_helper(
    state: &AppState,
    contract: FieldElement,
    entry_point: FieldElement,
    calldata: Vec<FieldElement>,
) -> Result<Vec<FieldElement>, String> {
    let result = state
        .provider
        .call(
            FunctionCall {
                contract_address: contract,
                entry_point_selector: entry_point,
                calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await;

    result.map_err(|e| format!("{}", e))
}
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    async fn inner(
        state: Arc<AppState>,
        query: VerifyQuery,
    ) -> Result<(StatusCode, Json<serde_json::Value>), String> {
        let task_id = 6;
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
            domain_res,
        )
        .await?;

        let mut twitter = false;
        let mut discord = false;
        for verifier_contract in &state.conf.starknetid_contracts.verifier_contracts {

            let twitter_verifier_data = call_contract_helper(
                &state,
                state.conf.starknetid_contracts.identity_contract,
                selector!("get_verifier_data"),
                vec![
                    id_res[0],
                    short_string!("twitter"),
                    *verifier_contract,
                    FieldElement::ZERO,
                ],
            )
            .await?;
        if twitter_verifier_data[0] != felt!("0") {
            twitter = true;
        }
    
            let discord_verifier_data = call_contract_helper(
                &state,
                state.conf.starknetid_contracts.identity_contract,
                selector!("get_verifier_data"),
                vec![
                    id_res[0],
                    short_string!("discord"),
                    *verifier_contract,
                    FieldElement::ZERO,
                ],
            )
            .await?;
        if discord_verifier_data[0] != felt!("0") {
            discord = true;
        }
        }
        

        if twitter && discord{
            match state.upsert_completed_task(query.addr, task_id).await {
                Ok(_) => Ok((StatusCode::OK, Json(json!({"res": true})))),
                Err(e) => Err(e.to_string()),
            }
        } else if !twitter{
            Err("You have not verified your Twitter account".to_string())
        } else {
            Err("You have not verified your Discord account".to_string())
        }
    }

    match inner(state, query).await {
        Ok(val) => val.into_response(),
        Err(err) => get_error(err),
    }
}
