use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use futures::TryStreamExt;
use mongodb::bson::{doc, Document, Bson};
use reqwest::StatusCode;
use std::sync::Arc;
use chrono::{Duration, Utc};

/*
 this endpoint will return static data for one address
 -> get total users
 Steps to get data over time:
 1) iterate over one week timestamps and add total points and get top 3 and get user position
 2) iterate over one month timestamps and add total points and get top 3 and get user position
 3) iterate over one year timestamps and add total points and get top 3 and get user position
 */


pub struct GetLeaderboardQuery {
    addr: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let users_collection = state.db.collection::<Document>("user_exp");

    // get total users
    let total_users_pipeline = vec![
        doc! {
            "$group": {
                "_id": "$address",
            }
        },
    ];

    let mut total_users: i64 = 0; // Initialize with a value

    match users_collection.aggregate(total_users_pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                total_users = total_users + 1;
            }
            (StatusCode::OK, Json(total_users)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }

    // iterate over weekly data
    let one_week_ago = Utc::now() - Duration::days(7);

    let weekly_pipeline = vec![
        doc! {
            "$match": {
            // Filter documents with a date field greater than or equal to one week ago
            "timestamp": {
                    "$gte": Bson::DateTime(DateTime::<Utc>::from(one_week_ago)) }
        }
        },
        doc! {
            "$group": {
                "_id": "$address",
                "total_points": {
                    "$sum": "$experience"
                }
            }
        },
    ];

    match users_collection.aggregate(weekly_pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                println!("weekly {}", result);
            }
            (StatusCode::OK, Json("hey")).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }


    // iterate over monthly data
    let one_month_ago = Utc::now() - Duration::days(30);
    let monthly_pipeline = vec![
        doc! {
            "$match": doc! {
            // Filter documents with a date field greater than or equal to one month ago
            "timestamp": doc!{
                    "$gte": Bson::DateTime(DateTime::<Utc>::from(one_month_ago)) }
        }
        },
        doc! {
            "$group": doc!{
                "_id": "$address",
                "total_points": doc!{
                    "$sum": "$experience"
                }
            }
        },
    ];

    match users_collection.aggregate(monthly_pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                println!("monthly {}", result);
            }
            (StatusCode::OK, Json("hey")).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }


    // iterate over all time data
    let all_time_pipeline = vec![
        doc! {
            "$group": doc! {
                "_id": "$address",
                "total_points": doc!{
                    "$sum": "$experience"
                }
            }
        },
    ];

    match users_collection.aggregate(all_time_pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                println!("all time {}", result);
            }
            (StatusCode::OK, Json("hey")).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
