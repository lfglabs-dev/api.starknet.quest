#[macro_use]
mod utils;
mod common;
mod config;
mod endpoints;
mod logger;
mod models;
mod middleware;

use crate::utils::{add_leaderboard_table, run_boosts_raffle};
use axum::{http::StatusCode, Router};
use axum_auto_routes::route;
use middleware::auth_middleware;
use mongodb::{bson::doc, options::ClientOptions, Client};
use reqwest::Url;
use serde_derive::Serialize;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};
use std::{borrow::Cow, sync::Arc};
use std::{net::SocketAddr, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};
use utils::WithState;


lazy_static::lazy_static! {
    pub static ref ROUTE_REGISTRY: Mutex<Vec<Box<dyn WithState>>> = Mutex::new(Vec::new());
}

#[derive(Serialize)]
struct LogData<'a> {
    token: &'a str,
    log: LogPayload<'a>,
}

#[derive(Serialize)]
struct LogPayload<'a> {
    app_id: &'a str,
    r#type: &'a str,
    message: Cow<'a, str>,
    timestamp: i64,
}

#[tokio::main]
async fn main() {
    let conf = config::load();
    let logger = logger::Logger::new(&conf.watchtower);

    logger.info(format!(
        "quest_server: starting v{}",
        env!("CARGO_PKG_VERSION")
    ));

    let client_options = ClientOptions::parse(&conf.database.connection_string)
        .await
        .unwrap();

    let shared_state = Arc::new(models::AppState {
        logger: logger.clone(),
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
        logger.async_severe("Unable to connect to database").await;
        return;
    } else {
        logger.info("Connected to database");
    }

    let db_instance = shared_state.db.clone();
    run_boosts_raffle(
        &db_instance,
        conf.quest_boost.update_interval,
        logger.clone(),
    );
    add_leaderboard_table(&shared_state.db).await;

    let cors = CorsLayer::new().allow_headers(Any).allow_origin(Any);
     // Admin router with middleware
    let admin_router = endpoints::admin::admin_routes()
        .layer(axum::middleware::from_fn(auth_middleware)); // Apply middleware to /admin routes

    // Main router without middleware
    let main_router = ROUTE_REGISTRY
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .fold(Router::new().with_state(shared_state.clone()), |acc, r| {
            acc.merge(r.to_router(shared_state.clone()))
        });

    // Combine the routers
    let app = Router::new()
        .nest("/admin", admin_router)
        .merge(main_router)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server.port));
    logger.info(format!(
        "server: listening on http://0.0.0.0:{}",
        conf.server.port
    ));
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