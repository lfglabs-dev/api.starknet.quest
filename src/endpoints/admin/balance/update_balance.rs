use crate::models::QuestTaskDocument;
use crate::{models::AppState, utils::get_error};
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;
use std::str::FromStr;
use std::sync::Arc;

// Define the request body structure
#[derive(Deserialize)]
pub struct CreateBalance {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    contracts: Option<String>,
    href: Option<String>,
    cta: Option<String>,
}

// Helper function to convert FieldElement to Bson
fn field_element_to_bson(fe: &FieldElement) -> mongodb::bson::Bson {
    mongodb::bson::Bson::String(fe.to_string())
}

// Define the route handler
pub async fn handler(
    State(state): State<Arc<AppState>>, // Extract state using Extension
    Json(body): Json<CreateBalance>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestTaskDocument>("tasks");
    // Filter to get existing quest
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
    if let Some(href) = &body.href {
        update_doc.insert("href", href);
    }
    if let Some(cta) = &body.cta {
        update_doc.insert("cta", cta);
    }
    if let Some(contracts) = &body.contracts {
        let parsed_contracts: Vec<FieldElement> = contracts
            .split(",")
            .map(|x| FieldElement::from_str(&x).unwrap())
            .collect();
        let contracts_bson: Vec<mongodb::bson::Bson> =
            parsed_contracts.iter().map(field_element_to_bson).collect();
        update_doc.insert("contracts", contracts_bson);
    }

    // Update quest query
    let update = doc! {
        "$set": update_doc
    };

    // Insert document into collection
    match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_) => get_error("Error updating tasks".to_string()),
    }
}

// Define the router for this module
// pub fn update_balance_router() -> Router {
//     Router::new()))
// }
