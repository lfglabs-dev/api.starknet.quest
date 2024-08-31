use axum::routing::Router;

// Import route modules
mod balance;
mod custom;
mod delete_task;
mod discord;
mod domain;
mod login;
mod nft_uri;
mod quest;
mod quest_boost;
mod quiz;
mod twitter;
mod user;

pub fn admin_routes() -> Router {
    Router::new()
        .nest("/balance/create", balance::create_balance::create_balance_router())
        .nest("/balance/update", balance::update_balance::update_balance_router())
        .nest("/custom/create", custom::create_custom::create_custom_router())
        .nest("/custom/update", custom::update_custom::update_custom_router())
        .nest("/discord/create", discord::create_discord::create_discord_router())
        .nest("/discord/update", discord::update_discord::update_discord_router())
        .nest("/domain/create", domain::create_domain::create_domain_router())
        .nest("/domain/update", domain::update_domain::update_domain_router())
        .nest("/create", nft_uri::create_uri::create_nft_uri_router())
        .nest("/get", nft_uri::get_nft_uri::get_nft_uri_router())
        .nest("/update", nft_uri::update_uri::update_nft_uri_router())
        .nest("/twitter/create", twitter::create_twitter_fw::create_twitter_router())
        .nest("/twitter/update", twitter::update_twitter_fw::update_twitter_router())


}
