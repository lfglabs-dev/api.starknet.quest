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

fn get_nimbora_strategy_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("angle".to_string(), "nstUSD".to_string());
    map.insert("pendle-puffer-eth".to_string(), "nppETH".to_string());
    map.insert("pendle-etherfi-eth".to_string(), "npeETH".to_string());
    map.insert("spark".to_string(), "nsDAI".to_string());
    map
}

#[route(get, "/discover/defi/get_alt_protocol_stats")]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let endpoint = &state.conf.discover.alt_protocols_api_endpoint;
    let client = reqwest::Client::new();
    let request_builder = client.get(endpoint);

    let nimbora_data = fetch_nimbora_data().await;
    let strategy_map = get_nimbora_strategy_map();

    let mut new_map = HashMap::new();
    match request_builder.send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Value::Object(ref map) = &json {
                    for (protocol, value) in map {
                        let mut mini_map = HashMap::new();
                        if let Value::Object(ref res) = value {
                            for (key, arr_value) in res {
                                if let Some(arr) = arr_value.as_array() {
                                    if !arr.is_empty() {
                                        mini_map.insert(key.clone(), arr.last().unwrap().clone());
                                    }
                                }
                            }
                            new_map.insert(protocol.clone(), mini_map);
                        }
                    }
                }

                // Update APRs with Nimbora data if available
                if let Ok(Value::Array(nimbora_strategies)) = nimbora_data {
                    for (_protocol, strategies) in new_map.iter_mut() {
                        for (strategy_name, strategy_data) in strategies.iter_mut() {
                            if let Some(nimbora_symbol) = strategy_map.get(strategy_name) {
                                if let Some(nimbora_strategy) = nimbora_strategies
                                    .iter()
                                    .find(|s| s["symbol"].as_str().unwrap_or("") == nimbora_symbol)
                                {
                                    if let Some(apr) = nimbora_strategy["apr"].as_str() {
                                        if let Ok(apr_value) = apr.parse::<f64>() {
                                            strategy_data["apr"] = Value::Number(
                                                serde_json::Number::from_f64(apr_value / 100.0)
                                                    .unwrap_or(serde_json::Number::from(0)),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    println!("Failed to fetch or parse Nimbora data");
                }

                return (StatusCode::OK, Json(new_map)).into_response();
            }
            Err(e) => get_error(format!("C - Try again later: {}", e)),
        },
        Err(e) => get_error(format!("B - Try again later: {}", e)),
    };

    get_error(format!("A - Try again later"))
}
