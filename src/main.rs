#[macro_use]
mod utils;
mod config;
mod endpoints;
mod models;
use axum::{http::StatusCode, routing::get, Router};
use mongodb::{bson::doc, options::ClientOptions, Client};
use reqwest::{Proxy, Url};
use starknet::providers::SequencerGatewayProvider;
use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    println!("quest_server: starting v{}", env!("CARGO_PKG_VERSION"));
    let conf = config::load();
    let client_options = ClientOptions::parse(&conf.database.connection_string)
        .await
        .unwrap();

    let client = match &conf.variables.proxy {
        Some(proxy_url) => reqwest::Client::builder().proxy(Proxy::http(proxy_url).unwrap()),
        None => reqwest::Client::builder(),
    }
    .build()
    .unwrap();

    let shared_state = Arc::new(models::AppState {
        conf: conf.clone(),
        provider: if conf.variables.is_testnet {
            SequencerGatewayProvider::new_with_client(
                Url::parse("https://alpha4.starknet.io/gateway").unwrap(),
                Url::parse("https://alpha4.starknet.io/feeder_gateway").unwrap(),
                client,
            )
        } else {
            SequencerGatewayProvider::new_with_client(
                Url::parse("https://alpha-mainnet.starknet.io/gateway").unwrap(),
                Url::parse("https://alpha-mainnet.starknet.io/feeder_gateway").unwrap(),
                client,
            )
        },
        db: Client::with_options(client_options)
            .unwrap()
            .database(&conf.database.name),
    });
    if shared_state
        .db
        .run_command(doc! {"ping": 1}, None)
        .await
        .is_err()
    {
        println!("error: unable to connect to database");
        return;
    } else {
        println!("database: connected")
    }

    let cors = CorsLayer::new().allow_headers(Any).allow_origin(Any);
    let app = Router::new()
        .route("/", get(root))
        .route("/quests/uri", get(endpoints::quests::uri::handler))
        .route(
            "/quests/contract_uri",
            get(endpoints::quests::contract_uri::handler),
        )
        .route("/get_quest", get(endpoints::get_quest::handler))
        .route("/get_quests", get(endpoints::get_quests::handler))
        .route("/get_tasks", get(endpoints::get_tasks::handler))
        .route(
            "/quests/starkfighter/claimable",
            get(endpoints::quests::starkfighter::claimable::handler),
        )
        .route(
            "/quests/starkfighter/verify_has_played",
            get(endpoints::quests::starkfighter::verify_has_played::handler),
        )
        .route(
            "/quests/starkfighter/verify_has_score_greater_than_50",
            get(endpoints::quests::starkfighter::verify_has_score_greater_than_50::handler),
        )
        .route(
            "/quests/starkfighter/verify_has_score_greater_than_100",
            get(endpoints::quests::starkfighter::verify_has_score_greater_than_100::handler),
        )
        .route(
            "/quests/starknetid/claimable",
            get(endpoints::quests::starknetid::claimable::handler),
        )
        .route(
            "/quests/starknetid/verify_has_domain",
            get(endpoints::quests::starknetid::verify_has_domain::handler),
        )
        .route(
            "/quests/starknetid/verify_twitter_follow",
            get(endpoints::quests::starknetid::verify_twitter_follow::handler),
        )
        .route(
            "/quests/starknetid/verify_has_root_domain",
            get(endpoints::quests::starknetid::verify_has_root_domain::handler),
        )
        .route(
            "/quests/starknetid/verify_socials",
            get(endpoints::quests::starknetid::verify_socials::handler),
        )
        .route(
            "/quests/jediswap/verify_has_root_domain",
            get(endpoints::quests::jediswap::verify_has_root_domain::handler),
        )
        .route(
            "/quests/jediswap/verify_added_liquidity",
            get(endpoints::quests::jediswap::verify_added_liquidity::handler),
        )
        .with_state(shared_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server.port));
    println!("server: listening on http://0.0.0.0:{}", conf.server.port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn root() -> (StatusCode, String) {
    (
        StatusCode::ACCEPTED,
        format!("quest_server v{}", env!("CARGO_PKG_VERSION")),
    )
}
