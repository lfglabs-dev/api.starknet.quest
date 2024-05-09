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
use axum_auto_routes::route;
use mongodb::bson::doc;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use starknet::core::types::FieldElement;

#[derive(Deserialize)]
pub struct TwitterOAuthCallbackQuery {
    code: String,
    state: FieldElement,
}

#[derive(Deserialize)]
pub struct Guild {
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[route(
    get,
    "/quests/ekubo/discord_fw_callback",
    crate::endpoints::quests::ekubo::discord_fw_callback
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TwitterOAuthCallbackQuery>,
) -> impl IntoResponse {
    let quest_id = 9;
    let task_id = 39;
    let guild_id = "1119209474369003600";
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
                "{}/quests/ekubo/discord_fw_callback",
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

    // Get user guild information
    let client = reqwest::Client::new();
    let response_result = client
        .get("https://discord.com/api/users/@me/guilds")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;
    let response: Vec<Guild> = match response_result {
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

    for guild in response {
        if guild.id == guild_id {
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
        }
    }

    get_error_redirect(
        error_redirect_uri,
        "You're not part of Ekubo's Discord server".to_string(),
    )
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
