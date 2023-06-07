use std::sync::Arc;

use crate::{
    models::{AppState, CompletedTasks},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, options::UpdateOptions};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct TwitterOAuthCallbackQuery {
    addr: String,
    code: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TwitterOAuthCallbackQuery>,
) -> impl IntoResponse {
    let task_id = 10;
    let addr = &query.addr;
    let authorization_code = &query.code;
    let jediswap_id = "1470315931142393857";

    // Exchange the authorization code for an access token
    let params = [
        ("client_id", &state.conf.twitter.oauth2_clientid),
        ("client_secret", &state.conf.twitter.oauth2_secret),
        ("code", &authorization_code.to_string()),
        (
            "redirect_uri",
            &format!(
                "http://127.0.0.1:8090/quests/jediswap/twitter_fw_callback?addr={}",
                addr
            ),
        ),
        ("code_verifier", &"NWIZBo0InJN7lmY_c".to_string()),
        ("grant_type", &"authorization_code".to_string()),
    ];
    let access_token = match exchange_authorization_code(params).await {
        Ok(token) => token,
        Err(e) => return get_error(format!("Failed to exchange authorization code: {}", e)),
    };

    // Get user information
    let url = "https://api.twitter.com/2/users/me";
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let client = reqwest::Client::new();
    let response_result = client.get(url).headers(headers).send().await;
    let response = match response_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => json,
                Err(e) => {
                    return get_error(format!(
                        "Failed to get JSON response while fetching user info: {}",
                        e
                    ))
                }
            }
        }
        Err(e) => return get_error(format!("Failed to send request to get user info: {}", e)),
    };
    let id = match response["data"]["id"].as_str() {
        Some(s) => s,
        None => return get_error("Failed to get 'id' from response data".to_string()),
    };

    // Check if user is following JediSwap
    let url_follower = format!("https://api.twitter.com/2/users/{}/following", id);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let client = reqwest::Client::new();
    let response_result = client.get(url_follower).headers(headers).send().await;
    let response = match response_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => json,
                Err(e) => {
                    return get_error(format!(
                        "Failed to get JSON response while fetching following: {}",
                        e
                    ))
                }
            }
        }
        Err(e) => {
            return get_error(format!(
                "Failed to send request to fetch user following: {}",
                e
            ))
        }
    };

    let following_ids = match response["data"].as_array() {
        Some(array) => array
            .iter()
            .map(|user| user["id"].as_str().unwrap().to_string())
            .collect::<Vec<String>>(),
        None => Vec::new(),
    };

    if following_ids.contains(&jediswap_id.to_string()) {
        // todo : update with latests changes on testnet branch
        let completed_tasks_collection = state.db.collection::<CompletedTasks>("completed_tasks");
        let filter = doc! { "address": addr, "task_id": task_id };
        let update = doc! { "$setOnInsert": { "address": addr, "task_id": task_id } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = completed_tasks_collection
            .update_one(filter, update, options)
            .await;

        match result {
            Ok(_) => (StatusCode::OK, Json(json!({"res": "task completed!"}))).into_response(),
            Err(e) => get_error(format!("{}", e)),
        }
    } else {
        get_error("You're not following Jediswap Twitter account'".to_string())
    }
}

async fn exchange_authorization_code(
    params: [(&str, &String); 6],
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.twitter.com/2/oauth2/token")
        .form(&params)
        .send()
        .await?;
    let json: serde_json::Value = res.json().await?;
    match json["access_token"].as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get 'access_token' from JSON response",
        ))),
    }
}
