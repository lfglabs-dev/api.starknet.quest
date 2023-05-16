use crate::models::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTItem {
    img: String,
    level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestDocument {
    id: u32,
    name: String,
    desc: String,
    issuer: String,
    category: String,
    rewards_endpoint: String,
    logo: String,
    rewards_img: String,
    rewards_title: String,
    rewards_nfts: Vec<NFTItem>,
}

#[derive(Serialize)]
pub struct QueryError {
    error: String,
}

async fn connect_to_database(state: Arc<AppState>) -> Collection<QuestDocument> {
    let client_options = ClientOptions::parse(&state.conf.database.connection_string)
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let database = client.database(&state.conf.database.name);
    let collection = database.collection::<QuestDocument>("quests");
    collection
}

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = connect_to_database(state.clone()).await;
    match collection.find_one(doc! {"id": query.id}, None).await {
        Ok(Some(quest)) => {
            let response = QuestDocument {
                id: quest.id,
                name: quest.name,
                desc: quest.desc,
                issuer: quest.issuer,
                category: quest.category,
                rewards_endpoint: quest.rewards_endpoint,
                logo: quest.logo,
                rewards_img: quest.rewards_img,
                rewards_title: quest.rewards_title,
                rewards_nfts: quest.rewards_nfts,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let error = QueryError {
                error: String::from("Quest not found"),
            };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
        Err(_) => {
            let error = QueryError {
                error: String::from("Error querying quest"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}
