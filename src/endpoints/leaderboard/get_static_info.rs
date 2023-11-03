/*
 this endpoint will return static data of leaderboard and position of user address
 Steps to get data over different time intervals :
 1) iterate over one week timestamps and add total points and get top 3 and get user position
 2) iterate over one month timestamps and add total points and get top 3 and get user position
 3) iterate over all timestamps and add total points and get top 3 and get user position
 */

use std::collections::HashMap;
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
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLeaderboardInfoQuery {
    addr: String,
}

pub async fn get_leaderboard_toppers(
    collection: &Collection<Document>,
    days: i64,
    address: &String,
) -> Document {
    let mut time_gap = 0;

    // get time gap
    if days > 0 {
        let gap_limit = Utc::now() - Duration::days(days);
        time_gap = gap_limit.timestamp_millis();
    }

    let leaderboard_pipeline = vec![
        doc! {
            "$match": doc! {
            // Filter documents with a date field greater than or equal to one month ago
            "timestamp": doc! {
                    "$gte": time_gap
                }
            }
        },
        doc! {
             "$sort" : doc! { "timestamp":-1}
        },
        doc! {
            "$group": doc!{
                "_id": "$address",
                "experience": doc!{
                    "$sum": "$experience"
                },
                "timestamp": doc! {
                    "$last": "$timestamp"
                }
            }
        },
        doc! {
            "$lookup": doc!{
                "from": "achieved",
                "localField": "_id",
                "foreignField": "addr",
                "as": "associatedAchievement"
            }
        },
        doc! {
            "$project": doc!{
                "_id": 0,
                "address": "$_id",
                "xp": "$experience",
                "achievements": doc!{
                    "$size": "$associatedAchievement"
                }
            }
        },
        doc! {
            "$sort": doc!{
                "xp": -1,
                "achievements": -1,
                "timestamp":1,
                "address":1,
            }
        },
        doc! {
            "$facet": doc! {
                "best_users": vec![
                    doc!{ "$limit": 3 }
                ],
                "totalUsers": vec![
                    doc!
                    {
                        "$count": "total"
                    }
                ],
                "rank": vec![
                    doc!{
          "$project": {
            "_id": 0,
            "address": "$address",
          },
        },
                ]
            }
        },
        doc! {
            "$project": doc!{
                "_id": 0,
                "totalUsers": 1,
                "best_users": 1,
                "rank": 1

                }
        },
        doc! {
            "$unwind": "$totalUsers",
        },
        doc! {
            "$project": doc!{
                "_id": 0,
                "length": "$totalUsers.total",
                "best_users": 1,
                "position": doc!{
                    "$add": [
          {
            "$indexOfArray": [
              "$rank.address",
              address,
            ],
          },
          1,
        ],
                }
                }
        },
    ];


    return match collection.aggregate(leaderboard_pipeline, None).await {
        Ok(mut cursor) => {
            let mut query_result = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                query_result.push(result)
            }
            query_result[0].clone()
        }
        Err(_) => Document::new(),
    };
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetLeaderboardInfoQuery>,
) -> impl IntoResponse {
    let addr: String = query.addr.to_string();
    let users_collection = state.db.collection::<Document>("user_exp");
    let weekly_toppers = get_leaderboard_toppers(&users_collection, 7, &addr).await;
    let monthly_toppers = get_leaderboard_toppers(&users_collection, 30, &addr).await;
    let all_time_toppers = get_leaderboard_toppers(&users_collection, -1, &addr).await;
    let mut res: HashMap<String, Document> = HashMap::new();

    res.insert("weekly".to_string(), weekly_toppers);
    res.insert("monthly".to_string(), monthly_toppers);
    res.insert("all_time".to_string(), all_time_toppers);
    (StatusCode::OK, Json(res)).into_response()
}
