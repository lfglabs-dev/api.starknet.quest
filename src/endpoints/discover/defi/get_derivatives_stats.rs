use crate::{
    models::{AppState},
    utils::get_error,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

#[route(get, "/discover/defi/get_derivatives_stats", crate::endpoints::discover::defi::get_derivatives_stats)]
pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let endpoint = &state.conf.discover.derivates_api_endpoint;
    let client = reqwest::Client::new();
    let request_builder = client.get(endpoint);

    let mut new_map = HashMap::new();
    match request_builder.send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Value::Object(ref map) = &json {
                    for key in map.keys() {
                        let arr_value = &map.get(key).unwrap().as_array();

                        let len = &arr_value.unwrap().len();
                        if len == &0 {
                            continue;
                        }
                        let last = arr_value.unwrap().get(len - 1).unwrap();
                        new_map.insert(key, last);
                    }
                }
                return (StatusCode::OK, Json(new_map)).into_response();
            }
            Err(_) => get_error(format!("Try again later")),
        }
        Err(_) => get_error(format!("Try again later")),
    };

    get_error(format!("Try again later"))
}
