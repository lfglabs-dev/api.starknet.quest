use std::sync::Arc;

use crate::{
    models::{AppState, VerifyAchievementQuery},
    utils::{get_error, to_hex, AchievementsTrait},
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

#[route(
    get,
    "/achievements/verify_avnu"
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyAchievementQuery>,
) -> impl IntoResponse {
    let addr = query.addr;
    if addr == FieldElement::ZERO {
        return get_error("Please connect your wallet first".to_string());
    }

    let achievement_id = query.id;
    if !(17..=19).contains(&achievement_id) {
        return get_error("Invalid achievement id".to_string());
    }

    let url = format!("https://starknet.api.avnu.fi/v1/takers/{}", to_hex(addr));
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(volume) = json["volumeInUSD"].as_f64() {
                    if (achievement_id == 17 && volume >= 500.0)
                        || (achievement_id == 18 && volume >= 5000.0)
                        || (achievement_id == 19 && volume >= 50000.0)
                    {
                        match state
                            .upsert_completed_achievement(addr, achievement_id)
                            .await
                        {
                            Ok(_) => {
                                (StatusCode::OK, Json(json!({"achieved": true}))).into_response()
                            }
                            Err(e) => get_error(format!("{}", e)),
                        }
                    } else {
                        get_error("Your volume on AVNU is too low".to_string())
                    }
                } else {
                    get_error("No data found for this address".to_string())
                }
            }
            Err(e) => get_error(format!(
                "Failed to get JSON response from Starkscan api: {}",
                e
            )),
        },
        Err(e) => get_error(format!("Failed to fetch Starkscan api: {}", e)),
    }
}
