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

/*
handle pagination

Scenarios:
input - 13
range - 9 -18

input -20
range - 15-25

input 18
range 13-23

input 25
range 20-30

Placing element in center =>
-> set range as (rank-((page_size/2)-1))) to (rank+page_size/2)
-> handle navigation  by adding the shift to the range and navigate backwards and forwards

*/

pub fn get_default_range(num: i64, page_size: i64, total_users: i64) -> (i64, i64) {
    let mut lower_range: i64 = 0;
    let mut upper_range: i64 = 0;

    // if rank is in top 5 then return default range
    if num <= page_size / 2 {
        lower_range = 1;
        upper_range = page_size;
    }

    // if rank is in bottom 5 then return default range
    else if num >= (total_users - page_size / 2) {
        lower_range = total_users - page_size;
        upper_range = total_users;
    }

    // if rank is in middle then return modified range where rank will be placed in middle of page
    else {
        lower_range = num - (page_size / 2 - 1);
        upper_range = match num + (page_size / 2) > total_users {
            true => total_users,
            false => num + (page_size / 2),
        };
    }

    return (lower_range, upper_range);
}


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
    page_size: i64,
    shift: i64,
    num: i64,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let address = query.addr.to_string();
    let page_size = query.page_size;
    let shift = query.shift;
    let num = query.num;
    let days = 7; //TODO: add dynamic days
    let total_users = 46;

    let mut lower_range: i64 = 0;
    let mut upper_range: i64 = 0;

    // get user position and get range to get page for showing user position
    if shift == 0 {
        (lower_range, upper_range) = get_default_range(num, page_size, total_users);
    }

    // get user position and set range if shift
    else {
        let (default_lower_range, default_upper_range) = get_default_range(num, page_size, total_users);

        /*
        -> calculate shift in elements needed.
        -> The sign is considered here so the value will be negative or positive depending on shift.
        -> If we want to move to next page then shift will be positive
        -> if we want to move to previous page then shift will be negative.
         */
        let shift_in_elements = shift * page_size;


        /*
        -> if lower range becomes negative then set it to 0
        -> if lower range becomes greater than total users then set it to total users - page_size to show last page.
        -> else set lower range to default lower range + shift in elements
         */
        if default_lower_range + shift_in_elements < 0 {
            lower_range = 0;
        } else if default_lower_range + shift_in_elements >= total_users {
            lower_range = total_users - page_size;
        } else {
            lower_range = default_lower_range + shift_in_elements;
        }

        /*
          set upper range
          -> if upper range becomes greater than total users then set it to total users
           -> else set upper range to lower range + shift in elements
         */
        upper_range = match lower_range + page_size > total_users {
            true => total_users,
            false => lower_range + page_size,
        };
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
                "includeArrayIndex": "rownum",

        },
        },
        doc! {
            "$addFields": {
            "docs.rank": {
                "$add": ["$rownum", 1],
            },
        },
        },
        doc! {
            "$replaceRoot": doc! {
            "newRoot": "$docs",
        }
        },
        doc! {
          "$match": doc!{
            "rank":doc!{
              "$gte":lower_range,
              "$lte":upper_range
            }
          }
        },
        doc! {
           "$project":{
                "_id":0,
                "address":"$_id",
                "total_points":1,
                "rank":1,
            }
        }
    ];


    match users_collection.aggregate(paginated_leaderboard_pipeline, None).await {
        Ok(mut cursor) => {
            let mut quest = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                quest.push(result);
            }
            (StatusCode::OK, Json(quest)).into_response()
        }
        Err(_) => get_error("Error querying quests".to_string()),
    }
}
