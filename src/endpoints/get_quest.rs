use crate::{
    models::{AppState, QuestDocument},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetQuestsQuery {
    id: u32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestsQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");
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
                img_card: quest.img_card,
                title_card: quest.title_card,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => get_error("Quest not found".to_string()),
        Err(_) => get_error("Error querying quest".to_string()),
    }
}
