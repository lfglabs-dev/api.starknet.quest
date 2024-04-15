use crate::models::{BoostTable, UpdateBoostQuery};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions};
use serde_json::json;
use std::sync::Arc;

#[route(put, "/admin/update_boost", crate::endpoints::admin::update_boost)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<UpdateBoostQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    // filter to get existing boost
    let filter = doc! {
        "quests": &body.quest_id,
    };
    let existing_boost = &collection.find_one(filter.clone(), None).await.unwrap();

    // create a boost if it does not exist
    if existing_boost.is_none() {
        // Get the last id in increasing order
        let last_id_filter = doc! {};
        let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
        let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

        let mut next_id = 1;
        if let Some(doc) = last_doc {
            let last_id = doc.id;
            next_id = last_id + 1;
        }

        let new_document = doc! {
            "name": &body.name,
            "img_url": &body.img_url,
            "amount": &body.amount,
            "token_decimals": &body.token_decimals,
            "token":&body.token,
            "expiry": &body.expiry,
            "num_of_winners": &body.num_of_winners,
            "quests": [&body.quest_id],
            "id": next_id,
            "hidden": false,
        };

        // insert document to boost collection
        return match collection
            .insert_one(from_document::<BoostTable>(new_document).unwrap(), None)
            .await
        {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({"message": "Boost created successfully"})).into_response(),
            )
                .into_response(),
            Err(e) => get_error("Error creating boosts".to_string()),
        };
    }

    // update boost
    let update = doc! {
        "$set": {
            "amount": &body.amount,
            "token": &body.token,
            "expiry": &body.expiry,
            "num_of_winners": &body.num_of_winners,
            "token_decimals": &body.token_decimals,
            "name": &body.name,
            "img_url": &body.img_url,
        }
    };
    let options = FindOneAndUpdateOptions::default();
    return match collection
        .find_one_and_update(filter, update, options)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        )
            .into_response(),
        Err(e) => get_error("error updating boost".to_string()),
    };
}
