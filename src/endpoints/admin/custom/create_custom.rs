use crate::models::{QuestDocument, QuestTaskDocument,JWTClaims};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc};
use mongodb::options::FindOneOptions;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::utils::verify_quest_auth;
use axum::http::HeaderMap;
use jsonwebtoken::{Validation,Algorithm,decode,DecodingKey};


pub_struct!(Deserialize; CreateCustom {
    quest_id: i64,
    name: String,
    desc: String,
    cta: String,
    href: String,
    api: String,
});

#[route(post, "/admin/tasks/custom/create", crate::endpoints::admin::custom::create_custom)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateCustom>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_doc = &collection.find_one(last_id_filter, options).await.unwrap();

    let quests_collection = state.db.collection::<QuestDocument>("quests");


    let res= verify_quest_auth(user, &quests_collection, &(body.quest_id as i64)).await;
    if !res {
        return get_error("Error creating task".to_string());
    };

    let mut next_id = 1;
    if let Some(doc) = last_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = QuestTaskDocument {
        name: body.name.clone(),
        desc: body.desc.clone(),
        verify_redirect: Some(body.href.clone()),
        href: body.href.clone(),
        quest_id : body.quest_id,
        id: next_id,
        cta: body.cta.clone(),
        verify_endpoint: body.api.clone(),
        verify_endpoint_type: "default".to_string(),
        task_type: Some("custom".to_string()),
        discord_guild_id: None,
        quiz_name: None,
    };

    // insert document to boost collection
    return match collection
        .insert_one(new_document,
            None,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error creating tasks".to_string()),
    };
}
