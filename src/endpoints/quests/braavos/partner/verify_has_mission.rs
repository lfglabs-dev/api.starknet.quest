use crate::{
    common::verify_has_root_or_braavos_domain::verify_has_root_or_braavos_domain,
    models::{AppState, VerifyQuery},
    utils::{get_error, get_error_redirect},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::{
    header::{self, AUTHORIZATION},
    StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DiscordOAuthCallbackQuery {
    code: String,
    state: FieldElement,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    id: String,
    #[allow(dead_code)]
    username: String,
    discriminator: String,
    global_name: Option<String>,
    avatar: Option<String>,
    bot: Option<bool>,
    system: Option<bool>,
    mfa_enabled: bool,
    banner: Option<String>,
    accent_color: Option<u32>,
    locale: Option<String>,
    verified: Option<bool>,
    email: Option<String>,
    flags: Option<u32>,
    premium_type: Option<u32>,
    public_flags: Option<u32>,
    avatar_decoration: Option<String>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscordOAuthCallbackQuery>,
) -> impl IntoResponse {
    let quest_id = 100;
    let task_id = 50;
    let authorization_code = &query.code;
    let error_redirect_uri = format!(
        "{}/quest/{}?task_id={}&res=false",
        state.conf.variables.app_link, quest_id, task_id
    );

    // Exchange the authorization code for an access token
    let params = [
        ("client_id", &state.conf.discord.oauth2_clientid),
        ("client_secret", &state.conf.discord.oauth2_secret),
        ("code", &authorization_code.to_string()),
        (
            "redirect_uri",
            &format!(
                "{}/quests/braavos/partner/verify_has_mission",
                state.conf.variables.api_link
            ),
        ),
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
    let client = reqwest::Client::new();
    let response_result = client
        .get("https://discord.com/api/users/@me")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;
    let response: DiscordUser = match response_result {
        Ok(response) => {
            let json_result = response.json().await;
            println!("json_result: {:?}", json_result);
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

    let discord_id = response.id;
    println!("discord_id: {:?}", discord_id);
    let username = response.username;
    println!("username: {:?}", username);

    // Get Crew3 profile from Discord username
    let url = format!(
        "https://api.zealy.io/communities/braavos/users?discordId={}",
        discord_id
    );
    let client = reqwest::Client::new();
    let response_result = client
        .get(url)
        .header(
            header::HeaderName::from_static("x-api-key"),
            header::HeaderValue::from_str(&state.conf.quests.braavos.crew3_api_key).unwrap(),
        )
        .send()
        .await;
    let response = match response_result {
        Ok(response) => {
            println!("response from Zealy: {:?}", response);
            let json_result = response.json::<serde_json::Value>().await.unwrap();
            println!("json_result from Zealy: {:?}", json_result);
            // match json_result {
            //     Ok(json) => json,
            //     Err(e) => {
            //         return get_error(format!(
            //             "Failed to get JSON response while fetching user info: {}",
            //             e
            //         ));
            //     }
            // }
            return (StatusCode::OK, Json(json!({"res": true}))).into_response();
        }
        Err(e) => {
            return get_error(format!("Failed to send request to fetch user info: {}", e));
        }
    };

    return (StatusCode::OK, Json(json!({"res": true}))).into_response();
}

async fn exchange_authorization_code(
    params: [(&str, &String); 5],
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await?;
    let json: serde_json::Value = res.json().await?;
    match json["access_token"].as_str() {
        Some(s) => Ok(s.to_string()),
        None => {
            println!(
                "Failed to get 'access_token' from JSON response : {:?}",
                json
            );
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to get 'access_token' from JSON response : {:?}",
                    json
                ),
            )))
        }
    }
}
