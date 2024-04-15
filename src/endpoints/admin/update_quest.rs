use crate::models::{ QuestDocument,  UpdateQuestQuery};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use serde_json::json;
use std::sync::Arc;


#[route(put, "/admin/update_quest", crate::endpoints::admin::update_boost)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateQuestQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");

    // filter to get existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let existing_quest = &collection.find_one(filter.clone(), None).await.unwrap();
    if existing_quest.is_none() {
        return get_error("quest does not exist".to_string());
    }


    // update quest query
    let update = doc! {
        "$set": {
            "name": &body.name,
            "desc": &body.desc,
            "expiry": &body.expiry,
            "disabled": &body.disabled,
            "start_time": &body.start_time,
        }
    };

    return match collection
        .find_one_and_update(filter, update, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        )
            .into_response(),
        Err(e) => get_error("error updating quest".to_string()),
    };
}
