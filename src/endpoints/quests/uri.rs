use crate::models::AppState;
use crate::utils::get_error;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct TokenURI {
    name: String,
    description: String,
    image: String,
    attributes: Option<Vec<Attribute>>,
}

#[derive(Serialize)]
pub struct Attribute {
    trait_type: String,
    value: u32,
}

#[derive(Deserialize)]
pub struct LevelQuery {
    level: Option<String>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(level_query): Query<LevelQuery>,
) -> Response {
    let level = level_query
        .level
        .and_then(|level_str| level_str.parse::<u32>().ok());

    fn get_level(level_int: u32) -> &'static str {
        match level_int {
            12 => "Chef",
            11 => "Officer",
            10 => "Soldier",
            2 => "Silver",
            3 => "Gold",
            _ => "Bronze",
        }
    }

    match level {
        Some(level_int) if level_int > 0 && level_int <= 3 => {
            let image_link = format!(
                "{}/starkfighter/level{}.webp",
                state.conf.variables.app_link, level_int
            );
            let response = TokenURI {
                name: format!("StarkFighter {} Arcade", get_level(level_int)),
                description: "A starknet.quest NFT won during the Starkfighter event.".into(),
                image: image_link,
                attributes: Some(vec![Attribute {
                    trait_type: "level".into(),
                    value: level_int,
                }]),
            };
            (StatusCode::OK, Json(response)).into_response()
        }

        Some(4) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet ID Tribe Totem".into(),
                description: "A Starknet Quest NFT won for creating a StarknetID profile.".into(),
                image: format!("{}/starknetid/nft1.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(5) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "JediSwap Light Saber".into(),
                description: "A JediSwap NFT won for interacting with the protocol.".into(),
                image: format!("{}/jediswap/padawan.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(6) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "AVNU Astronaut".into(),
                description: "An AVNU NFT won for interacting with the protocol.".into(),
                image: format!("{}/avnu/astronaut.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(7) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Sithswap Helmet".into(),
                description: "A Sithswap NFT won for interacting with the protocol.".into(),
                image: format!(
                    "{}/sithswap/sith_helmet.webp",
                    state.conf.variables.app_link
                ),
                attributes: None,
            }),
        )
            .into_response(),

        Some(8) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Zklend Artemis".into(),
                description: "A Zklend NFT won for interacting with the protocol.".into(),
                image: format!("{}/zklend/artemis.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(9) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Stark Tribe Shield".into(),
                description: "A Starknet Quest NFT won for showing allegiance to the Stark Tribe."
                    .into(),
                image: format!("{}/starknetid/shield.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(level_int) if level_int > 9 && level_int <= 12 => {
            let image_link = format!(
                "{}/starknetid/necklace{}.webp",
                state.conf.variables.app_link,
                level_int - 9
            );
            let response = TokenURI {
                name: format!("Starknet ID {} Necklace", get_level(level_int)),
                description: "A Starknet Quest NFT won during a Starknet ID quest.".into(),
                image: image_link,
                attributes: Some(vec![Attribute {
                    trait_type: "level".into(),
                    value: level_int,
                }]),
            };
            (StatusCode::OK, Json(response)).into_response()
        }

        Some(13) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "StarkOrb".into(),
                description: "An Orbiter NFT won for interacting with the protocol.".into(),
                image: format!("{}/orbiter/orbiter.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(14) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Ekubo".into(),
                description: "An Ekubo NFT won for interacting with the protocol.".into(),
                image: format!("{}/ekubo/concentration.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(15) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Carmine".into(),
                description: "A Carmine NFT won for interacting with the protocol.".into(),
                image: format!("{}/carmine/specialist.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(16) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Morphine".into(),
                description: "A Morphine NFT won for interacting with the protocol.".into(),
                image: format!("{}/morphine/yielder.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(17) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "MySwap".into(),
                description: "A MySwap NFT won for interacting with the protocol.".into(),
                image: format!("{}/myswap/LP.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(18) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet Pro Score x Starknet ID Quest NFT".into(),
                description: "This Starknet commemorative Non-Fungible Token represents the first step into the Starknet universe. By getting a Stark domain name and becoming a Whisperer of Braavos, you are building solid foundations for your Starknet experience.".into(),
                image: format!("{}/braavos/starknetid.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(19) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet Giga Brain NFT".into(),
                description: "A Starknet Giga Brain NFT won for successfuly responding to a quiz.".into(),
                image: format!("{}/starknet/gigabrain.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(20) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Account Abstraction Mastery NFT".into(),
                description: "An Account Abstraction Mastery NFT won for successfully responding to a quiz.".into(),
                image: format!("{}/starknet/aa.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(21) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "The Focus Tree".into(),
                description: "The Focus Tree NFT won during a Starknet Quest.".into(),
                image: format!("{}/focustree/focustree.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(22) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "The Element Gemstone".into(),
                description: "An Element Gemstone NFT won for successfully finishing the Quest".into(),
                image: format!("{}/element/elementGem.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(23) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "The Briq Element Gemstone".into(),
                description: "A Briq Element Gemstone NFT won for successfully finishing the Quest".into(),
                image: format!("{}/element/briqGem.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(24) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "The Layerswap Element Gemstone".into(),
                description: "A Layerswap Element Gemstone NFT won for successfully finishing the Quest".into(),
                image: format!("{}/element/layerswapGem.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),


        Some(25) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "The Starknet.id Element Gemstone".into(),
                description: "A Starknet.id Element Gemstone NFT won for successfully finishing the Quest".into(),
                image: format!("{}/element/starknetidGem.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(26) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet Pro Score x mySwap Quest NFT".into(),
                description: "This Starknet commemorative Non-Fungible Token represents the first steps into the Starknet universe. By using mySwap and becoming a Whisperer of Braavos, you are building solid foundations for your Starknet experience.".into(),
                image: format!("{}/braavos/myswap.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(27) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Nostra - LaFamiglia Rose".into(),
                description: "A Nostra - LaFamiglia Rose NFT won for successfully finishing the Quest".into(),
                image: format!("{}/nostra/rose.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),

        Some(28) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet Pro Score x AVNU Quest NFT".into(),
                description: "This Starknet commemorative Non-Fungible Token represents the first steps into the Starknet universe. By using AVNU you are building solid foundations for your Starknet experience.".into(),
                image: format!("{}/braavos/avnu.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),
      
        Some(29) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Starknet Pro Score x Braavos Wallet Quest NFT".into(),
                description: "This Starknet commemorative Non-Fungible Token represents the first steps into the Starknet universe. By using Braavos Wallet and becoming a Whisperer of Braavos, you are building solid foundations for your Starknet experience.".into(),
                image: format!("{}/braavos/wallet.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        )
            .into_response(),
      
      
        Some(30) => (
            StatusCode::OK,
            Json(TokenURI {
                name: "Rango Exchange Quest NFT".into(),
                description: "A Rango Exchange Quest NFT won for successfully finishing the Quest".into(),
                image: format!("{}/rango/bridge.webp", state.conf.variables.app_link),
                attributes: None,
            }),
        ).into_response(),
    
        _ => get_error("Error, this level is not correct".into()),
    }
}
