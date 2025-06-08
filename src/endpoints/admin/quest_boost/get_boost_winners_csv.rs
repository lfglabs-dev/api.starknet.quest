use crate::middleware::auth::auth_middleware;
use crate::models::{AppState, BoostTable};
use crate::utils::get_error;
use axum::{
    extract::{Extension, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetBoostWinnersCsvParams {
    boost_id: i64,
}

#[route(get, "/admin/boosts/get_boost_winners_csv", auth_middleware)]
pub async fn get_boost_winners_csv_handler(
    State(state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>,
    Query(params): Query<GetBoostWinnersCsvParams>,
) -> impl IntoResponse {
    let collection = state.db.collection::<BoostTable>("boosts");

    let filter = doc! { "id": params.boost_id };

    match collection.find_one(filter, None).await {
        Ok(Some(boost_doc)) => {
            // Convert winners data to CSV format
            match create_csv_response(&boost_doc.winner, params.boost_id) {
                Ok(csv_content) => {
                    let headers = [
                        (header::CONTENT_TYPE, "text/csv"),
                        (
                            header::CONTENT_DISPOSITION,
                            &format!("attachment; filename=\"boost_{}_winners.csv\"", params.boost_id),
                        ),
                    ];

                    (StatusCode::OK, headers, csv_content).into_response()
                }
                Err(e) => get_error(format!("Error creating CSV: {}", e)),
            }
        }
        Ok(None) => get_error(format!("Boost with id {} not found", params.boost_id)),
        Err(e) => get_error(format!("Error fetching boost winners: {}", e)),
    }
}

pub(crate) fn create_csv_response(
    winners: &Option<Vec<String>>,
    boost_id: i64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut csv_content = String::new();
    
    // Add CSV header
    csv_content.push_str("boost_id,winner_address,position\n");
    
    // Handle the Option<Vec<String>> format
    match winners {
        Some(winner_vec) => {
            for (index, winner_address) in winner_vec.iter().enumerate() {
                csv_content.push_str(&format!("{},{},{}\n", boost_id, winner_address, index + 1));
            }
        }
        None => {
            // No winners found, just return header
        }
    }
    
    Ok(csv_content)
}