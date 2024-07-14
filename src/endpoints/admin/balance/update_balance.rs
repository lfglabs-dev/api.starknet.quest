use crate::models::{JWTClaims, QuestTaskDocument};
use crate::utils::verify_task_auth;
use crate::{models::AppState, utils::get_error};
use axum::http::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;
use std::str::FromStr;
use std::sync::Arc;

pub_struct!(Deserialize; CreateBalance {
    id: i64,
    name: Option<String>,
    desc: Option<String>,
    contracts: Option<String>,
    href: Option<String>,
    cta: Option<String>,
});

// Helper function to convert FieldElement to Bson
fn field_element_to_bson(fe: &FieldElement) -> mongodb::bson::Bson {
    mongodb::bson::Bson::String(fe.to_string())
}

#[route(
    post,
    "/admin/tasks/balance/update",
    crate::endpoints::admin::balance::update_balance
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<CreateBalance>,
) -> impl IntoResponse {
    let user = check_authorization!(headers, &state.conf.auth.secret_key.as_ref()) as String;
    let collection = state.db.collection::<QuestTaskDocument>("tasks");

    let res = verify_task_auth(user, &collection, &(body.id as i32)).await;
    if !res {
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

    // update quest query
    let update = doc! {
        "$set": update_doc
    };

    // insert document to boost collection
    return match collection.find_one_and_update(filter, update, None).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Task updated successfully"})).into_response(),
        )
            .into_response(),
        Err(_e) => get_error("Error updating tasks".to_string()),
    };
}
