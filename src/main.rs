#[macro_use]
mod utils;
mod common;
mod config;
mod endpoints;
mod models;

use axum::{http::StatusCode, Router};
use axum_auto_routes::route;
use mongodb::{bson::doc, options::ClientOptions, Client};
use reqwest::Url;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};
use std::sync::Arc;
use std::{net::SocketAddr, sync::Mutex};
use utils::WithState;

use crate::utils::{add_leaderboard_table, run_boosts_raffle};
use tower_http::cors::{Any, CorsLayer};

lazy_static::lazy_static! {
    pub static ref ROUTE_REGISTRY: Mutex<Vec<Box<dyn WithState>>> = Mutex::new(Vec::new());
}

#[tokio::main]
async fn main() {
    println!("quest_server: starting v{}", env!("CARGO_PKG_VERSION"));
    let conf = config::load();
    let client_options = ClientOptions::parse(&conf.database.connection_string)
        .await
        .unwrap();

    let shared_state = Arc::new(models::AppState {
        conf: conf.clone(),
        provider: JsonRpcClient::new(HttpTransport::new(
            Url::parse(&conf.variables.rpc_url).unwrap(),
        )),
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
        println!("database: connected");
    }

    let db_instance = shared_state.db.clone();
    run_boosts_raffle(&db_instance, conf.quest_boost.update_interval);
    add_leaderboard_table(&shared_state.db).await;

    let cors = CorsLayer::new().allow_headers(Any).allow_origin(Any);
    let app = ROUTE_REGISTRY.lock().unwrap().clone().into_iter().fold(
        Router::new().with_state(shared_state.clone()).layer(cors),
        |acc, r| acc.merge(r.to_router(shared_state.clone())),
    );
    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server.port));
    println!("server: listening on http://0.0.0.0:{}", conf.server.port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[route(get, "/")]
async fn root() -> (StatusCode, String) {
    (
        StatusCode::ACCEPTED,
        format!("quest_server v{}", env!("CARGO_PKG_VERSION")),
    )
}
