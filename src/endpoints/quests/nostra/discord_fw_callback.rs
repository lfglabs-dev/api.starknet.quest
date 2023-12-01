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
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use starknet::core::types::FieldElement;

#[derive(Deserialize)]
pub struct DiscordOAuthCallbackQuery {
    code: String,
    state: FieldElement,
}

#[derive(Deserialize, Debug)]
pub struct Guild {
    id: String,
    #[allow(dead_code)]
    name: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscordOAuthCallbackQuery>,
) -> impl IntoResponse {
    let quest_id = 20;
    let task_id = 80;
    let guild_id = "1002209435868987463";
    let authorization_code = &query.code;
    let error_redirect_uri = format!(
        "{}/quest/{}?task_id={}&res=false",
        state.conf.variables.app_link, quest_id, task_id
    );

    println!("Authorization code: {}", authorization_code);


    // Exchange the authorization code for an access token
    let params = [
        ("client_id", &state.conf.discord.oauth2_clientid),
        ("client_secret", &state.conf.discord.oauth2_secret),
        ("code", &authorization_code.to_string()),
        (
            "redirect_uri",
            &format!(
                "{}/quests/nostra/discord_fw_callback",
                state.conf.variables.api_link
            ),
        ),
        ("grant_type", &"authorization_code".to_string()),
    ];

    println!("Params: {:?}", params);

    let access_token = match exchange_authorization_code(params).await {
        Ok(token) => {
            println!("Access token: {}", token);
            token
        },
        Err(e) => {
            println!("Failed to exchange authorization code: {}", e);
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
    println!("Response result guild: {:?}", response_result);
    let response: Vec<Guild> = match response_result {
        Ok(response) => {
            let json_result = response.json().await;
            println!("JSON result guild: {:?}", json_result);
            match json_result {
                Ok(json) => {
                    println!("JSON: {:?}", json);
                    json
                },
                Err(e) => {
                    println!("Failed to get JSON response while fetching user info: {}", e);
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
            println!("Failed to send request to get user info response : {}", e);
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to send request to get user info: {}", e),
            );
        }
    };

    for guild in response {
        println!("Guild: {:?}", guild);
        if guild.id == guild_id {
            match state.upsert_completed_task(query.state, task_id).await {
                Ok(_) => {
                    let redirect_uri = format!(
                        "{}/quest/{}?task_id={}&res=true",
                        state.conf.variables.app_link, quest_id, task_id
                    );
                    return success_redirect(redirect_uri);
                }
                Err(e) => {
                    println!("Guild Error: {}", e);
                    return get_error_redirect(error_redirect_uri, format!("{}", e))
                },
            }
        }
    }

    get_error_redirect(
        error_redirect_uri,
        "You're not part of Nostra's Discord server".to_string(),
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

    println!("Response: {:?}", res);
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
