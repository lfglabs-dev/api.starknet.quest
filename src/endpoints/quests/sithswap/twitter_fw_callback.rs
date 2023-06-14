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
    let quest_id = 5;
    let task_id = 21;
    let addr_str = FieldElement::to_string(&query.addr);
    let authorization_code = &query.code;
    let sithswap_id = "1494635968141398043";
    let error_redirect_uri = format!(
        "{}/quest/{}?task_id={}&res=false",
        state.conf.variables.app_link, quest_id, task_id
    );

    // Exchange the authorization code for an access token
    let params = [
        ("client_id", &state.conf.twitter.oauth2_clientid),
        ("client_secret", &state.conf.twitter.oauth2_secret),
        ("code", &authorization_code.to_string()),
        (
            "redirect_uri",
            &format!(
                "{}/quests/sithswap/twitter_fw_callback?addr={}",
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
            );
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
            );
        }
    };
    let id = match response["data"]["id"].as_str() {
        Some(s) => s,
        None => {
            return get_error_redirect(
                error_redirect_uri,
                "Failed to get 'id' from response data".to_string(),
            );
        }
    };

    // Check if user is following JediSwap
    let url_follower_base = format!("https://api.twitter.com/2/users/{}/following", id);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let client = reqwest::Client::new();

    let mut following_ids = Vec::new();
    let mut next_token = None;
    loop {
        let url_follower = match &next_token {
            Some(token) => format!("{}?pagination_token={}", url_follower_base, token),
            None => url_follower_base.clone(),
        };

        let response_result = client
            .get(&url_follower)
            .headers(headers.clone())
            .send()
            .await;
        let response = match response_result {
            Ok(response) => {
                let json_result = response.json::<serde_json::Value>().await;
                match json_result {
                    Ok(json) => json,
                    Err(e) => {
                        return get_error_redirect(
                            error_redirect_uri,
                            format!(
                                "Failed to get JSON response while fetching following: {}",
                                e
                            ),
                        );
                    }
                }
            }
            Err(e) => {
                return get_error_redirect(
                    error_redirect_uri,
                    format!("Failed to send request to fetch user following: {}", e),
                );
            }
        };

        let ids: Vec<String> = response["data"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|user| user["id"].as_str().unwrap().to_string())
            .collect();

        following_ids.extend(ids);

        next_token = response["meta"]["next_token"]
            .as_str()
            .map(|s| s.to_string());

        if next_token.is_none() || following_ids.contains(&sithswap_id.to_string()) {
            break;
        }
    }

    if following_ids.contains(&sithswap_id.to_string()) {
        match state.upsert_completed_task(query.addr, task_id).await {
            Ok(_) => {
                let redirect_uri = format!(
                    "{}/quest/{}?task_id={}&res=true",
                    state.conf.variables.app_link, quest_id, task_id
                );
                success_redirect(redirect_uri)
            }
            Err(e) => get_error_redirect(error_redirect_uri, format!("{}", e)),
        }
    } else {
        get_error_redirect(
            error_redirect_uri,
            "You're not following Jediswap Twitter account".to_string(),
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
            "Failed to get 'access_token' from JSON response",
        ))),
    }
}
