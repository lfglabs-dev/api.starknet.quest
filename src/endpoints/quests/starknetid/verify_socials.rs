use crate::{
    models::{AppState, CompletedTasks, VerifyQuery},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, options::UpdateOptions};
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

        let discord_verifier_data = call_contract_helper(
            &state,
            state.conf.starknetid_contracts.identity_contract,
            selector!("get_verifier_data"),
            vec![
                id_res.result[0],
                short_string!("discord"),
                state.conf.starknetid_contracts.verifier_contract,
            ],
        )
        .await?;

        if twitter_verifier_data.result[0] != felt!("0")
            && discord_verifier_data.result[0] != felt!("0")
        {
            let completed_tasks_collection =
                state.db.collection::<CompletedTasks>("completed_tasks");
            let filter = doc! { "address": addr.to_string(), "task_id": task_id };
            let update =
                doc! { "$setOnInsert": { "address": addr.to_string(), "task_id": task_id } };
            let options = UpdateOptions::builder().upsert(true).build();

            let _ = completed_tasks_collection
                .update_one(filter, update, options)
                .await
                .map_err(|e| format!("{}", e))?;
            Ok((StatusCode::OK, Json(json!({"res": true}))))
        } else if twitter_verifier_data.result[0] == felt!("0") {
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
