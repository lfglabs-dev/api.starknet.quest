use crate::{
    models::AppState,
    utils::{get_error, get_error_redirect, success_redirect, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use reqwest::header::{self, AUTHORIZATION};
use serde::Deserialize;
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
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscordOAuthCallbackQuery>,
) -> impl IntoResponse {
    let quest_id = 100;
    let task_id = 50;
    let mission_id = "2e1a9301-14da-4430-9ae0-b617e1c379f4";
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

    // Get Zealy profile from Discord id
    let discord_id = response.id;
    let url = format!(
        "https://api.zealy.io/communities/braavos/users?discordId={}",
        discord_id
    );
    let client = reqwest::Client::new();
    let response_result = client
        .get(url)
        .header(
            header::HeaderName::from_static("x-api-key"),
            header::HeaderValue::from_str(&state.conf.quests.braavos.api_key_user).unwrap(),
        )
        .send()
        .await;
    let zealy_response = match response_result {
        Ok(response) => {
            let json_result = response.json::<serde_json::Value>().await;
            match json_result {
                Ok(json) => json,
                Err(e) => {
                    return get_error_redirect(
                        error_redirect_uri,
                        format!("Failed to get User info from Zealy's response: {:?}", e),
                    );
                }
            }
        }
        Err(e) => {
            return get_error(format!("Failed to send request to fetch user info: {}", e));
        }
    };
    if let Some(id) = zealy_response.get("id") {
        let zealy_id = id.as_str().unwrap();

        // Get user completed mission from Zealy API
        let url = format!(
            "https://api.zealy.io/communities/braavos/claimed-quests?user_id={}&quest_id={}",
            zealy_id, mission_id
        );
        let client = reqwest::Client::new();
        let mission_response = client
            .get(url)
            .header(
                header::HeaderName::from_static("x-api-key"),
                header::HeaderValue::from_str(&state.conf.quests.braavos.api_key_claimed_mission)
                    .unwrap(),
            )
            .send()
            .await;
        let missions = match mission_response {
            Ok(response) => {
                let json_result = response.json::<serde_json::Value>().await;
                match json_result {
                    Ok(json) => json,
                    Err(e) => {
                        return get_error_redirect(
                            error_redirect_uri,
                            format!("Failed to get user's mission from Zealy: {:?}", e),
                        );
                    }
                }
            }
            Err(e) => {
                return get_error(format!(
                    "Failed to send request to fetch user missions: {}",
                    e
                ));
            }
        };
        if let Some(mission_array) = missions.get("data").and_then(|v| v.as_array()) {
            if !mission_array.is_empty() {
                match state.upsert_completed_task(query.state, task_id).await {
                    Ok(_) => {
                        let redirect_uri = format!(
                            "{}/quest/{}?task_id={}&res=true",
                            state.conf.variables.app_link, quest_id, task_id
                        );
                        return success_redirect(redirect_uri);
                    }
                    Err(e) => return get_error_redirect(error_redirect_uri, format!("{}", e)),
                }
            } else {
                return get_error_redirect(
                    error_redirect_uri,
                    "You have not fulfilled this mission on Zealy".to_string(),
                );
            }
        } else {
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to get Zealy ID from response: {:?}", zealy_response),
            );
        }
    } else {
        return get_error_redirect(
            error_redirect_uri,
            format!("Failed to get Zealy ID from response: {:?}", zealy_response),
        );
    };
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
