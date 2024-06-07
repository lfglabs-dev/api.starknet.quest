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
use serde::Deserialize;
use crate::models::{QuestTaskDocument,JWTClaims};
use crate::utils::verify_task_auth;
use axum::http::HeaderMap;
use jsonwebtoken::{Validation,Algorithm,decode,DecodingKey};


pub_struct!(Deserialize; CreateTwitterFw {
    name: Option<String>,
    desc: Option<String>,
    id: i32,
});

#[route(post, "/admin/tasks/domain/update", crate::endpoints::admin::domain::update_domain)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateTwitterFw>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref())  as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");


    let res= verify_task_auth(user,  &collection,&body.id).await;
    if !res{
        return get_error("Error updating tasks".to_string());
    }


    // filter to get existing quest
    let filter = doc! {
        "id": &body.id,
    };

    let mut update_doc = doc! {};

    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }

    // update quest query
    let update = doc! {
        "$set": update_doc
    };


    // insert document to boost collection
    return match collection
        .find_one_and_update(filter, update, None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error updating tasks".to_string()),
    };
}
