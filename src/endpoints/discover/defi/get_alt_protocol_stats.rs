use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

async fn fetch_nimbora_data() -> Result<Value, reqwest::Error> {
    let nimbora_endpoint = "https://stats.nimbora.io/yield-dex/strategies";
    let client = reqwest::Client::new();
    client.get(nimbora_endpoint).send().await?.json().await
}

#[route(get, "/discover/defi/get_alt_protocol_stats")]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let endpoint = &state.conf.discover.alt_protocols_api_endpoint;
    let client = reqwest::Client::new();
    let request_builder = client.get(endpoint);

    let nimbora_data = fetch_nimbora_data().await;

    let mut new_map = HashMap::new();
    match request_builder.send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Value::Object(ref map) = &json {
                    for key in map.keys() {
                        let mut mini_map = HashMap::new();
                        let value = &map.get(key).unwrap();
                        if let Value::Object(ref res) = value {
                            for key in res.keys() {
                                let arr_value = &res.get(key).unwrap().as_array();

                                let len = &arr_value.unwrap().len();
                                if len == &0 {
                                    continue;
                                }
                                let last = arr_value.unwrap().get(len - 1).unwrap();
                                mini_map.insert(key.clone(), last.clone());
                            }
                            new_map.insert(key.clone(), mini_map);
                        }
                    }
                }

                // Update APRs with Nimbora data if available
                if let Ok(nimbora_value) = nimbora_data {
                    if let Value::Array(nimbora_strategies) = nimbora_value {
                        for (_protocol, strategies) in new_map.iter_mut() {
                            let updated_strategies = strategies
                                .iter()
                                .map(|(strategy_name, strategy_data)| {
                                    let mut updated_data = strategy_data.clone();
                                    if let Some(nimbora_strategy) =
                                        nimbora_strategies.iter().find(|s| {
                                            s["name"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_lowercase()
                                                .contains(&strategy_name.to_lowercase())
                                        })
                                    {
                                        println!(
                                            "Matching Nimbora strategy found for {}",
                                            strategy_name
                                        );
                                        if let (Some(base_apr), Some(incentives_apr)) = (
                                            nimbora_strategy["aprBreakdown"]["base"].as_str(),
                                            nimbora_strategy["aprBreakdown"]["incentives"].as_str(),
                                        ) {
                                            let base_apr: f64 = base_apr.parse().unwrap_or(0.0);
                                            let incentives_apr: f64 =
                                                incentives_apr.parse().unwrap_or(0.0);
                                            let total_apr = base_apr + incentives_apr;

                                            updated_data["apr"] = Value::Number(
                                                serde_json::Number::from_f64(total_apr)
                                                    .unwrap_or(serde_json::Number::from(0)),
                                            );
                                        }
                                    } else {
                                        println!(
                                            "No matching Nimbora strategy found for {}",
                                            strategy_name
                                        );
                                    }
                                    (strategy_name.clone(), updated_data)
                                })
                                .collect();
                            *strategies = updated_strategies;
                        }
                    } else {
                        println!("Nimbora data is not in the expected format");
                    }
                } else {
                    println!("Failed to fetch Nimbora data");
                }

                return (StatusCode::OK, Json(new_map)).into_response();
            }
            Err(e) => get_error(format!("C - Try again later: {}", e)),
        },
        Err(e) => get_error(format!("B - Try again later: {}", e)),
    };

    get_error(format!("A - Try again later"))
}
