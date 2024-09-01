use axum::routing::Router;

// Import route modules
mod balance;
mod custom;
pub mod delete_task;
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
        //balance
        .nest("/tasks/balance", balance::create_balance::create_balance_router())
        .nest("/tasks/balance", balance::update_balance::update_balance_router())
        
        //custom
        .nest("/tasks/custom", custom::create_custom::create_custom_router())
        .nest("/tasks/custom", custom::update_custom::update_custom_router())
        
        //discord
        .nest("/tasks/discord", discord::create_discord::create_discord_router())
        .nest("/tasks/discord", discord::update_discord::update_discord_router())
        
        //domain
        .nest("/tasks/domain", domain::create_domain::create_domain_router())
        .nest("/tasks/domain", domain::update_domain::update_domain_router())
        
        //nft_uri
        .nest("/tasks/nft_uri", nft_uri::create_uri::create_nft_uri_router())
        .nest("/tasks/nft_uri", nft_uri::get_nft_uri::get_nft_uri_router())
        .nest("/tasks/nft_uri", nft_uri::update_uri::update_nft_uri_router())

        //quest
        .nest("/tasks/quest", quest::create_quest::create_quest_router())
        .nest("/tasks/quest", quest::get_quest::get_quest_routes())
        .nest("/tasks/quest", quest::get_quests::get_quests_routes())
        .nest("/tasks/quest", quest::get_tasks::get_tasks_routes())
        .nest("/tasks/quest", quest::update_quest::update_quest_routes())

        //quest_boost
        .nest("/tasks/quest_boost", quest_boost::create_boost::create_boost_router())
        .nest("/tasks/quest_boost", quest_boost::update_boost::update_boost_router())

        //quiz
        .nest("/tasks/quiz", quiz::create_question::create_question_routes())
        .nest("/tasks/quiz", quiz::update_question::update_question_routes())
        .nest("/tasks/quiz", quiz::create_quiz::create_quiz_routes())
        .nest("/tasks/quiz", quiz::get_quiz::get_quiz_routes())
        .nest("/tasks/quiz", quiz::update_quiz::update_quiz_routes())


        //twitter_fw
        .nest("/task/twitter_fw", twitter::create_twitter_fw::create_twitter_fw_router())
        .nest("/task/twitter_fw", twitter::update_twitter_fw::update_twitter_fw_router())
        
        //twitter_rw
        .nest("/task/twitter_rw", twitter::create_twitter_rw::create_twitter_rw_router())
        .nest("/task/twitter_rw", twitter::update_twitter_rw::update_twitter_rw_router())

        //user
        .nest("/user", user::create_user::create_user_routes())

        //task deletion
        .nest("/task", delete_task::delete_task_routes())



}
