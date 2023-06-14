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
use starknet::core::types::{BlockId, CallFunction, FieldElement};
use starknet::id::decode;
use starknet::macros::selector;
use starknet::providers::Provider;

#[derive(Deserialize)]
pub struct TwitterOAuthCallbackQuery {
    addr: FieldElement,
    code: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TwitterOAuthCallbackQuery>,
) -> impl IntoResponse {
    let task_id = 30;
    let quest_id = 7;
    let addr_str = FieldElement::to_string(&query.addr);
    let authorization_code = &query.code;
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
                "{}/quests/twitter_tribe/twitter_name_callback?addr={}",
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

    // get starkname from address
    let call_result = state
        .provider
        .call_contract(
            CallFunction {
                contract_address: state.conf.starknetid_contracts.naming_contract,
                entry_point_selector: selector!("address_to_domain"),
                calldata: vec![query.addr],
            },
            BlockId::Latest,
        )
        .await;

    let starkname = match call_result {
        Ok(result) => {
            let name_len: u32 = result.result[0].try_into().unwrap();
            if name_len == 1 {
                format!("{}.stark", decode(result.result[1]))
            } else {
                format!(
                    "{}.{}.stark",
                    decode(result.result[1]),
                    decode(result.result[2])
                )
            }
            // decode(result.result[1]);

            // let mut name = String::new();
            // // Iterate from 1 to name_len (inclusive) and decode each element
            // for i in 1..=name_len {
            //     name.push_str(&decode(result.result[FieldElement::from(i)]));
            // }
            // name
        }
        Err(e) => {
            return get_error_redirect(
                error_redirect_uri,
                format!("Failed to get starkname from address: {}", e),
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
    let twitter_name = match response["data"]["name"].as_str() {
        Some(s) => s,
        None => {
            return get_error_redirect(
                error_redirect_uri,
                "Failed to get 'name' from response data".to_string(),
            );
        }
    };
    if twitter_name
        .to_lowercase()
        .contains(starkname.to_lowercase().as_str())
    {
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
            format!(
                "Twitter name {} does not contain {}",
                twitter_name,
                starkname.as_str()
            ),
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
