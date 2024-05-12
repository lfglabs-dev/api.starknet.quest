use crate::{models::AppState, utils::get_error};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use mongodb::bson::{doc, from_document};
use mongodb::options::{FindOneOptions};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;
use crate::models::QuestTaskDocument;

pub_struct!(Deserialize; CreateQuiz {
    name: String,
    desc: String,
    help_link: String,
    cta: String,
    intro: String,
    quest_id: i32,
});

#[route(post, "/admin/tasks/quiz/create", crate::endpoints::admin::quiz::create_quiz)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    body: Json<CreateQuiz>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let quiz_collection = state.db.collection::<QuestTaskDocument>("quizzes");

    // Get the last id in increasing order
    let last_id_filter = doc! {};
    let options = FindOneOptions::builder().sort(doc! {"id": -1}).build();
    let last_quiz_doc = &quiz_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();

    let mut next_quiz_id = 1;
    if let Some(doc) = last_quiz_doc {
        let last_id = doc.id;
        next_quiz_id = last_id + 1;
    }

    let new_quiz_document = doc! {
            "name": &body.name,
            "desc": &body.desc,
            "id": next_quiz_id,
            "intro" : &body.intro,
    };

    match quiz_collection
        .insert_one(from_document::<QuestTaskDocument>(new_quiz_document).unwrap(), None)
        .await
    {
        Ok(res) => res,
        Err(_e) => return get_error("Error creating quiz".to_string()),
    };


    let last_task_doc = &tasks_collection.find_one(last_id_filter.clone(), options.clone()).await.unwrap();
    let mut next_id = 1;
    if let Some(doc) = last_task_doc {
        let last_id = doc.id;
        next_id = last_id + 1;
    }

    let new_document = doc! {
            "name": &body.name,
            "desc": &body.desc,
            "href": &body.help_link,
            "cta": &body.cta,
            "quest_id" : &body.quest_id,
            "id": next_id,
            "verify_endpoint": "/quests/verify_quiz",
            "verify_endpoint_type": "quiz",
            "quiz_name": next_quiz_id,
        };

    return  match tasks_collection
        .insert_one(from_document::<QuestTaskDocument>(new_document).unwrap(), None)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Boost created successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => return get_error("Error creating quiz".to_string()),
    };
}
