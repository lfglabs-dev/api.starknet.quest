use std::sync::Arc;

use crate::utils::fetch_json_from_url;
use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error, to_hex, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use serde_json::json;
use starknet::core::types::FieldElement;

#[route(get, "/quests/element/briq/verify_own_briq")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 68;
    if query.addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let url = format!(
        "https://api.briq.construction/v1/user/data/starknet-mainnet-dojo/{}",
        to_hex(query.addr)
    );
    match fetch_json_from_url(url).await {
        Ok(response) => {
            if let Some(sets) = response.get("sets") {
                match sets {
                    serde_json::Value::Array(sets_array) => {
                        for set in sets_array.iter() {
                            if let serde_json::Value::String(set_str) = set {
                                let url = format!(
                                    "https://api.briq.construction/v1/metadata/starknet-mainnet-dojo/{}",
                                    set_str
                                );
                                match fetch_json_from_url(url).await {
                                    Ok(metadata_response) => {
                                        if let Some(_properties) =
                                            metadata_response.get("properties")
                                        {
                                            match state
                                                .upsert_completed_task(query.addr, task_id)
                                                .await
                                            {
                                                Ok(_) => {
                                                    return (
                                                        StatusCode::OK,
                                                        Json(json!({"res": true})),
                                                    )
                                                        .into_response();
                                                }
                                                Err(e) => {
                                                    return get_error(format!("{}", e));
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => return get_error(e),
                                }
                            }
                        }
                    }
                    _ => {
                        return get_error("No Briq sets founds".to_string());
                    }
                }
            }
            get_error("No Briq sets founds".to_string())
        }
        Err(e) => get_error(e),
    }
}
