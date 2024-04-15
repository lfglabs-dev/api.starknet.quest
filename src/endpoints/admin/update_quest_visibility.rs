use crate::models::{BoostTable};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::{FindOneAndUpdateOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct UpdateQuestVisibility{
    id: u32,
}

#[route(post, "/admin/update_quest_visibility", crate::endpoints::admin::update_quest_visibility)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateQuestVisibility>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    // filter to get existing boost
    let filter = doc! {
        "quests": &body.id,
    };
    // check if boost needs to be disabled
    let update = doc! {
            "$set": {
                "hidden": true,
            }
        };
    let options = FindOneAndUpdateOptions::default();
    match collection
        .find_one_and_update(filter, update, options)
        .await
    {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(json!({"message": "disabled successfully"})),
            )
                .into_response();
        }
        Err(e) => return get_error("error removing boost".to_string()),
    }
}
