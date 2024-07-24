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

type StrkFarmAPIResponse = serde_json::Value;

#[route(
    get,
    "/quests/strkFarm/check_balance",
    crate::endpoints::quests::strk_farm::check_balance
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let task_id = 185;
    let addr = &query.addr;
    let url = format!("https://www.strkfarm.xyz/api/stats/{addr}");
    let res = reqwest::get(&url).await.unwrap().text().await.unwrap();
    // Res in a JSON containing the user's balance
    let json: StrkFarmAPIResponse = serde_json::from_str(&res).unwrap();
    let usd = json["holdingsUSD"].as_f64().unwrap();
    if usd == 0.0 {
        get_error("You didn't invest on StrkFarm.".to_string())
    } else if usd < 10.0 {
        get_error(format!(
            "You need to invest at least $10 on StrkFarm (You have ${}).",
            usd
        ))
    } else {
        match state.upsert_completed_task(query.addr, task_id).await {
            Ok(_) => (StatusCode::OK, Json(json!({"res": true}))).into_response(),
            Err(e) => get_error(format!("{}", e)),
        }
    }
}
