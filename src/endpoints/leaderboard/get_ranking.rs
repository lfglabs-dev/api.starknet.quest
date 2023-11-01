/*
 this endpoint will return leaderboard ranking for one address
-> get user position
-> get leaderboard
1) iterate over one week timestamps and add total points
2) get full rankings for one week
3) get the page requested with formula of ((user position)/page_size) +1
4) fetch previous and next page depending on request.
5) add flag value called "last index": to fetch the next set of documents from the position last index
5) "last index" can also act as a flag value to check if there is a previous page or next page
(last_index === -1 then no previous page && last_index === total.documents.length then no next page)
 */

// TODO: get paginated data

use crate::{models::AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use mongodb::bson::{doc, Document, Bson};
use reqwest::StatusCode;
use std::sync::Arc;
use chrono::{Duration, Utc};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use crate::utils::get_error;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCompletedQuestsQuery {
    addr: FieldElement,
    page_size: i32,
    shift: i32,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let page_size = query.page_size;
    let shift = query.shift;
    let days = 7; //TODO: add dynamic days

    let total_users = 46;
    let mut total_pages = total_users / page_size;
    if (total_users % page_size != 0) {
        total_pages = (total_users / page_size) + 1;
    }
    let users_collection = state.db.collection::<Document>("user_exp");

    let mut time_gap = 0;

    // get time gap
    if (days > 0) {
        let gap_limit = Utc::now() - Duration::days(days);
        time_gap = gap_limit.timestamp_millis();
    }

    let collect_documents_within_timeframe = doc! {
            "$match": doc! {
            // Filter documents with a date field greater than or equal to one month ago
            "timestamp": doc!{
                    "$gte": time_gap
                }
            }
        };

    let group_users_by_points_pipeline =
        doc! {
            "$group": doc!{
                "_id": "$address",
                "total_points": doc!{
                    "$sum": "$experience"
                },
                "timestamp": {
                    "$last": "$timestamp"
                }
            }
            };

    let sort_documents_in_descending_order = doc! { "$sort" : doc! { "total_points" : -1 ,"timestamp":1} };

    // let user_rank_pipeline = vec![
    //     collect_documents_within_timeframe,
    //     group_users_by_points_pipeline,
    //     sort_documents_in_descending_order,
    //     doc! {
    //         "$group": {
    //         "_id": null,
    //         "docs": doc! {
    //             "$push": "$$ROOT",
    //         },
    //     },
    //     },
    //     doc! {
    //         "$unwind": doc! {
    //         "path": "$docs",
    //         "includeArrayIndex": "rownum",
    //     },
    //     },
    //     doc! {
    //         "$match": doc! {
    //         "docs._id":
    //         address,
    //     },
    //     },
    //     doc! {
    //         "$addFields": doc! {
    //         "docs.rank": doc! {
    //             "$add": ["$rownum", 1],
    //         },
    //     },
    //     },
    //     doc! {
    //         "$replaceRoot": doc! {
    //         "newRoot": "$docs",
    //     }
    //     },
    // ];

    let paginated_leaderboard_pipeline = [
        collect_documents_within_timeframe,
        group_users_by_points_pipeline,
        sort_documents_in_descending_order,
        doc! {
            "$group": {
            "_id": null,
            "docs": doc! {
                "$push": "$$ROOT",
            },
        },
        },
        doc! {
            "$unwind": doc! {
            "path": "$docs",
        },
        },
        doc! {
            "$replaceRoot": doc! {
            "newRoot": "$docs",
        }
        },
    ];


    match users_collection.aggregate(paginated_leaderboard_pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(result) = cursor.try_next().await.unwrap() {
                println!("result: {}", result);
            }
            (StatusCode::OK, Json("ehy")).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
