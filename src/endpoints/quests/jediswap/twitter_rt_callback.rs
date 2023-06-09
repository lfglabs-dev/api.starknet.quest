use std::sync::Arc;

use crate::utils::CompletedTasksTrait;
use crate::{
    models::AppState,
    utils::{get_error_redirect, success_redirect},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use mongodb::bson::doc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use starknet::core::types::FieldElement;

#[derive(Deserialize)]
pub struct TwitterOAuthCallbackQuery {
    addr: FieldElement,
    code: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TwitterOAuthCallbackQuery>,
) -> impl IntoResponse {
    let task_id = 11;
    let addr_str = FieldElement::to_string(&query.addr);
    let authorization_code = &query.code;
    let error_redirect_uri = format!(
        "{}/quest/2?task_id={}&res=false",
        state.conf.variables.app_link, task_id
    );

    // Exchange the authorization code for an access token
    let params = [
        ("client_id", &state.conf.twitter.oauth2_clientid),
        ("client_secret", &state.conf.twitter.oauth2_secret),
        ("code", &authorization_code.to_string()),
        (
            "redirect_uri",
            &format!(
                "{}/quests/jediswap/twitter_rt_callback?addr={}",
                state.conf.variables.api_link, addr_str
            ),
        ),
        ("code_verifier", &"NWIZBo0InJN7lmY_c".to_string()),
        ("grant_type", &"authorization_code".to_string()),
    ];
    let access_token = match exchange_authorization_code(params).await {
        Ok(token) => token,
        Err(e) => {
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to exchange authorization code: {}", e),
            )
        }
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
                    return get_error_redirect(
                        error_redirect_uri,
                        format!(
                            "Failed to get JSON response while fetching user info: {}",
                            e
                        ),
                    );
                }
            }
        }
        Err(e) => {
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to send request to get user info: {}", e),
            )
        }
    };
    let id = match response["data"]["id"].as_str() {
        Some(s) => s,
        None => {
            return get_error_redirect(
                error_redirect_uri,
                "Failed to get 'id' from response data".to_string(),
            )
        }
    };

    // Check if user has retweeted tweet
    let url_retweeted = format!(
        "https://api.twitter.com/2/tweets/{}/retweeted_by",
        state.conf.quests.jediswap.tweet_id
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let client = reqwest::Client::new();
    let response_result = client.get(url_retweeted).headers(headers).send().await;
    let response = match response_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => json,
                Err(e) => {
                    return get_error_redirect(
                        error_redirect_uri,
                        format!("Failed to send request to get user info: {}", e),
                    )
                }
            }
        }
        Err(e) => {
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to send request to fetch tweet info: {}", e),
            )
        }
    };

    let reteweeted_ids = match response["data"].as_array() {
        Some(array) => array
            .iter()
            .map(|user| user["id"].as_str().unwrap().to_string())
            .collect::<Vec<String>>(),
        None => Vec::new(),
    };

    if reteweeted_ids.contains(&id.to_string()) {
        match state.upsert_completed_task(query.addr, task_id).await {
            Ok(_) => success_redirect(format!(
                "{}/quest/2?task_id={}&res=true",
                state.conf.variables.app_link, task_id
            )),
            Err(e) => get_error_redirect(error_redirect_uri, format!("{}", e)),
        }
    } else {
        get_error_redirect(
            error_redirect_uri,
            "You have not retweeted the Quest thread yet.'".to_string(),
        )
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
            format!("Failed to get 'access_token' from JSON response: {}", json),
        ))),
    }
}
