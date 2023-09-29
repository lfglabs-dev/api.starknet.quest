#[macro_use]
mod utils;
mod common;
mod config;
mod endpoints;
mod models;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
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
        .route("/get_quiz", get(endpoints::get_quiz::handler))
        .route("/get_quest", get(endpoints::get_quest::handler))
        .route("/get_quests", get(endpoints::get_quests::handler))
        .route(
            "/get_trending_quests",
            get(endpoints::get_trending_quests::handler),
        )
        .route("/get_tasks", get(endpoints::get_tasks::handler))
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
            get(endpoints::quests::starknetid::verify_twitter_fw::handler),
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
        .route(
            "/quests/jediswap/verify_twitter_fw",
            get(endpoints::quests::jediswap::verify_twitter_fw::handler),
        )
        .route(
            "/quests/jediswap/verify_twitter_rt",
            get(endpoints::quests::jediswap::verify_twitter_rt::handler),
        )
        .route(
            "/quests/jediswap/claimable",
            get(endpoints::quests::jediswap::claimable::handler),
        )
        .route(
            "/quests/zklend/verify_has_root_domain",
            get(endpoints::quests::zklend::verify_has_root_domain::handler),
        )
        .route(
            "/quests/zklend/verify_borrow",
            get(endpoints::quests::zklend::verify_borrow::handler),
        )
        .route(
            "/quests/zklend/verify_twitter_fw",
            get(endpoints::quests::zklend::verify_twitter_fw::handler),
        )
        .route(
            "/quests/zklend/verify_twitter_rt",
            get(endpoints::quests::zklend::verify_twitter_rt::handler),
        )
        .route(
            "/quests/zklend/claimable",
            get(endpoints::quests::zklend::claimable::handler),
        )
        .route(
            "/quests/avnu/verify_twitter_rt",
            get(endpoints::quests::avnu::verify_twitter_rt::handler),
        )
        .route(
            "/quests/avnu/discord_fw_callback",
            get(endpoints::quests::avnu::discord_fw_callback::handler),
        )
        .route(
            "/quests/avnu/verify_swap",
            get(endpoints::quests::avnu::verify_swap::handler),
        )
        .route(
            "/quests/avnu/claimable",
            get(endpoints::quests::avnu::claimable::handler),
        )
        .route(
            "/quests/tribe/verify_has_domain",
            get(endpoints::quests::tribe::verify_has_domain::handler),
        )
        .route(
            "/quests/tribe/verify_has_root_domain",
            get(endpoints::quests::tribe::verify_has_root_domain::handler),
        )
        .route(
            "/quests/tribe/verify_three_years_expiry",
            get(endpoints::quests::tribe::verify_three_years_expiry::handler),
        )
        .route(
            "/quests/tribe/claimable",
            get(endpoints::quests::tribe::claimable::handler),
        )
        .route(
            "/quests/sithswap/verify_has_root_domain",
            get(endpoints::quests::sithswap::verify_has_root_domain::handler),
        )
        .route(
            "/quests/sithswap/verify_twitter_fw",
            get(endpoints::quests::sithswap::verify_twitter_fw::handler),
        )
        .route(
            "/quests/sithswap/verify_twitter_rt",
            get(endpoints::quests::sithswap::verify_twitter_rt::handler),
        )
        .route(
            "/quests/sithswap/verify_added_liquidity",
            get(endpoints::quests::sithswap::verify_added_liquidity::handler),
        )
        .route(
            "/quests/sithswap/claimable",
            get(endpoints::quests::sithswap::claimable::handler),
        )
        .route(
            "/quests/orbiter/verify_has_root_domain",
            get(endpoints::quests::orbiter::verify_has_root_domain::handler),
        )
        .route(
            "/quests/orbiter/verify_twitter_fw",
            get(endpoints::quests::orbiter::verify_twitter_fw::handler),
        )
        .route(
            "/quests/orbiter/verify_twitter_fw_sq",
            get(endpoints::quests::orbiter::verify_twitter_fw_sq::handler),
        )
        .route(
            "/quests/orbiter/verify_twitter_rt",
            get(endpoints::quests::orbiter::verify_twitter_rt::handler),
        )
        .route(
            "/quests/orbiter/verify_has_bridged",
            get(endpoints::quests::orbiter::verify_has_bridged::handler),
        )
        .route(
            "/quests/orbiter/claimable",
            get(endpoints::quests::orbiter::claimable::handler),
        )
        .route(
            "/quests/ekubo/claimable",
            get(endpoints::quests::ekubo::claimable::handler),
        )
        .route(
            "/quests/ekubo/discord_fw_callback",
            get(endpoints::quests::ekubo::discord_fw_callback::handler),
        )
        .route(
            "/quests/ekubo/verify_quiz",
            post(endpoints::quests::ekubo::verify_quiz::handler),
        )
        .route(
            "/quests/ekubo/verify_added_liquidity",
            get(endpoints::quests::ekubo::verify_added_liquidity::handler),
        )
        .route(
            "/quests/carmine/verify_quiz",
            post(endpoints::quests::carmine::verify_quiz::handler),
        )
        .route(
            "/quests/carmine/claimable",
            get(endpoints::quests::carmine::claimable::handler),
        )
        .route(
            "/quests/morphine/verify_quiz",
            post(endpoints::quests::morphine::verify_quiz::handler),
        )
        .route(
            "/quests/morphine/verify_added_liquidity",
            get(endpoints::quests::morphine::verify_added_liquidity::handler),
        )
        .route(
            "/quests/morphine/claimable",
            get(endpoints::quests::morphine::claimable::handler),
        )
        .route(
            "/quests/myswap/verify_added_liquidity",
            get(endpoints::quests::myswap::verify_added_liquidity::handler),
        )
        .route(
            "/quests/myswap/discord_fw_callback",
            get(endpoints::quests::myswap::discord_fw_callback::handler),
        )
        .route(
            "/quests/myswap/claimable",
            get(endpoints::quests::myswap::claimable::handler),
        )
        .route(
            "/achievements/verify_default",
            get(endpoints::achievements::verify_default::handler),
        )
        .route(
            "/achievements/verify_briq",
            get(endpoints::achievements::verify_briq::handler),
        )
        .route(
            "/achievements/verify_has_domain",
            get(endpoints::achievements::verify_has_domain::handler),
        )
        .route(
            "/achievements/fetch",
            get(endpoints::achievements::fetch::handler),
        )
        .route(
            "/achievements/fetch_buildings",
            get(endpoints::achievements::fetch_buildings::handler),
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
