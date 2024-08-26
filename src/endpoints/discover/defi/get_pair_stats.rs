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

#[route(get, "/discover/defi/get_pair_stats")]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let endpoint = &state.conf.discover.pairs_api_endpoint;
    let client = reqwest::Client::new();
    let request_builder = client.get(endpoint);

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
                                mini_map.insert(key, last);
                            }
                            new_map.insert(key, mini_map.clone());
                            mini_map.clear();
                        }
                    }
                }
                return (StatusCode::OK, Json(new_map)).into_response();
            }
            Err(_) => get_error(format!("Try again later")),
        },
        Err(_) => get_error(format!("Try again later")),
    };

    get_error(format!("Try again later"))
}
