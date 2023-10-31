use std::result;
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
use mongodb::Collection;

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

pub async fn get_total_users(
    collection: &Collection<Document>,
) -> Bson {
    // get total users
    let total_users_pipeline = vec![
        doc! {
            "$group": {
                "_id": "$address",
            }
        },
        doc! { "$count": "total_users" },
    ];

    match collection.aggregate(total_users_pipeline, None).await {
        Ok(mut cursor) => {
            let mut total_users: Bson = Bson::Null;
            while let Some(result) = &cursor.try_next().await.unwrap() {
                total_users = Bson::from(result.get("total_users").unwrap());
            }
            return total_users;
        }
        Err(_) => return Bson::Null,
    }
}


pub async fn get_leaderboard_toppers(
    collection: &Collection<Document>,
    days: i64
) -> Bson {
    let mut time_gap= 0;

    // get time gap
    if (days > 0){
        let gap_limit = Utc::now() - Duration::days(days);
        time_gap = gap_limit.timestamp_millis();
    }

    let leaderboard_pipeline = vec![
        doc! {
            "$match": doc! {
            // Filter documents with a date field greater than or equal to one month ago
            "timestamp": doc!{
                    "$gte": time_gap
                }
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
        doc! { "$sort" : doc! { "total_points" : -1 } },
        doc! { "$limit": 3 },
    ];


    match collection.aggregate(leaderboard_pipeline, None).await {
        Ok(mut cursor) => {
            let mut query_result = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                query_result.push(result)
            }
            return query_result.into();
        }
        Err(_) => return Bson::Null,
    }
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let users_collection = state.db.collection::<Document>("user_exp");

    let total_users = get_total_users(&users_collection).await;
    let weekly_toppers = get_leaderboard_toppers(&users_collection, 7).await;
    let monthly_toppers = get_leaderboard_toppers(&users_collection, 30).await;
    let all_time_toppers = get_leaderboard_toppers(&users_collection, -1).await;


    println!("total_users: {:?}", total_users);
    println!("weekly_toppers: {:?}", weekly_toppers);
    println!("monthly_toppers: {:?}", monthly_toppers);
    println!("all_time_toppers: {:?}", all_time_toppers);

    match all_time_toppers {
        Ok(mut cursor) => {
            let mut all_time: Vec<Document> = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                all_time.push(result).unwrap();
            }
            (StatusCode::OK, Json(all_time)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
