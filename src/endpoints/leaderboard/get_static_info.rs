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
 Steps to get data over time:
 1) iterate over one week timestamps and add total points and get top 3 and get user position
 2) iterate over one month timestamps and add total points and get top 3 and get user position
 3) iterate over one year timestamps and add total points and get top 3 and get user position
 */

// TODO: get user position in all 3 cases
pub struct GetLeaderboardQuery {
    addr: String,
}

pub async fn get_leaderboard_toppers(
    collection: &Collection<Document>,
    days: i64,
) -> Bson {
    let mut time_gap = 0;

    // get time gap
    if (days > 0) {
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
            // facet will allow us to run multiple pipelines on the same set of documents
            "$facet": {

                //sorting the users and getting the top 3
            "best_users": vec![
            doc!{ "$sort": doc!{ "experience": -1 } },
            doc!{ "$limit": 3 },
            ],

                //getting the total number of users
            "totalUsers": vec![doc!{ "$count": "total" }],
        },
        },
        doc! {
            "$unwind": "$totalUsers",
        },
        doc! {
            "$project": {
            "_id": 0,
            "totalUsers": "$totalUsers.total",
            "best_users": 1,
        },
        },
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

    let weekly_toppers = get_leaderboard_toppers(&users_collection, 7).await;
    let monthly_toppers = get_leaderboard_toppers(&users_collection, 30).await;
    let all_time_toppers = get_leaderboard_toppers(&users_collection, -1).await;


    println!("weekly_toppers: {:?}", weekly_toppers);
    println!("monthly_toppers: {:?}", monthly_toppers);
    println!("all_time_toppers: {:?}", all_time_toppers);


    (StatusCode::OK, Json("hey")).into_response()
}
