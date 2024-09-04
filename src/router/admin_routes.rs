use std::sync::Arc;
use axum::{routing::{post,get}, body, Router};
use crate::models::AppState;
use crate::endpoints::admin;
use crate::middleware::auth;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
use axum::middleware;

// Router for admin actions
pub fn routes() -> axum::Router<Arc<AppState>> {
    Router::new()
        //balance
        .route("/tasks/balance/create", post(admin::balance::create_balance::handler))
        .route("/tasks/balance/update", post(admin::balance::update_balance::handler))
        
        //custom
        .route("/tasks/custom/create", post(admin::custom::create_custom::handler))
        .route("/tasks/custom/update", post(admin::custom::update_custom::handler))
        
        //discord
        .route("/tasks/discord/create", post(admin::discord::create_discord::handler))
        .route("/tasks/discord/update", post(admin::discord::update_discord::handler))
        
        //domain
        .route("/tasks/domain/create", post(admin::domain::create_domain::handler))
        .route("/tasks/domain/update", post(admin::domain::update_domain::handler))
        
        //nft_uri
        .route("/nft_uri/create", post(admin::nft_uri::create_uri::handler))
        .route("/nft_uri/get_nft_uri", get(admin::nft_uri::get_nft_uri::handler))
        .route("/nft_uri/update", post(admin::nft_uri::update_uri::handler))
        
        //quest
        .route("/quest/create", post(admin::quest::create_quest::handler))
        .route("/quest/get_quest", get(admin::quest::get_quest::handler))
        .route("/quest/get_quests", get(admin::quest::get_quests::handler))
        .route("/quest/get_tasks", get(admin::quest::get_tasks::handler))
        .route("/quest/update", post(admin::quest::update_quest::handler))
        
        //quest_boost
        .route("/quest_boost/create_boost", post(admin::quest_boost::create_boost::handler))
        .route("/quest_boost/update_boost", post(admin::quest_boost::update_boost::handler))
        
        //quiz
        .route("/tasks/quiz/question/create", post(admin::quiz::create_question::handler))
        .route("/tasks/quiz/create", post(admin::quiz::create_quiz::handler))
        .route("/quiz/get_quiz", get(admin::quiz::get_quiz::handler))
        .route("/tasks/quiz/question/update", post(admin::quiz::update_question::handler))
        .route("/tasks/quiz/update", post(admin::quiz::update_quiz::handler))

        //twitter
        .route("/tasks/twitter_fw/create", post(admin::twitter::create_twitter_fw::handler))
        .route("/tasks/twitter_fw/update", post(admin::twitter::update_twitter_fw::handler))
        .route("/tasks/twitter_rw/create", post(admin::twitter::create_twitter_rw::handler))
        .route("/tasks/twitter_rw/update", post(admin::twitter::update_twitter_rw::handler))
        
        //user
        .route("/user/create", post(admin::user::create_user::handler))
        
        //delete_task
        .route("tasks/remove_task", post(admin::delete_task::handler))

        // middleware
        .layer(
            ServiceBuilder::new()
                .map_request_body(body::boxed)
                .layer(middleware::from_fn(auth::auth_middleware)),
        )

    }
