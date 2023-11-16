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
main page range - 9 -18

input -20
main page range - 15-25

input 18
main page range 13-23

input 25
main page range 20-30

Placing element in center =>
-> set range as (rank-((page_size/2)-1))) to (rank+page_size/2)
-> handle navigation  by adding the (shift*page_size) to the lower range and navigate backwards and forwards
*/

use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use mongodb::bson::{doc, Document};
use mongodb::Collection;
use reqwest::StatusCode;
use std::sync::Arc;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};


pub async fn get_user_rank(collection: &Collection<Document>, address: &String, start_timestamp: &i64, end_timestamp: &i64) -> Document {
    let user_rank_pipeline = vec![
        doc! {
            "$match": doc!{
                "timestamp": doc!{
                    "$gte": start_timestamp,
                    "$lte": end_timestamp
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
                "experience": 1,
                "achievements": doc!{
                    "$size": "$associatedAchievement"
                }
            }
        },
        doc! {
            "$sort": doc!{
                "experience": -1,
                "achievements": -1,
                "timestamp":1,
                "address":1,
            }
        },
        doc! {
           "$group": doc! {
            "_id": null,
            "addressList": { "$push": "$address" },
        },
        },
        doc! {
            "$project": doc!{
            "_id": 0,
            "addressList": 1,
        },
        },
        doc! {
            "$facet": {
            "total_users": vec! [
            doc!{
                "$project": doc! {
                "total": {
                    "$size": "$addressList",
                },
            },
            },
            ],
            "user_rank": vec![
            doc!{
                "$project": {
                "rank": {
                    "$add": [
                    doc! {
                        "$indexOfArray": [
                        "$addressList",
                        address,
                        ],
                    },
                    1,
                    ],
                },
            },
            },
            ],
        },
        },
        doc! {
            "$project": doc!{
            "total_users": doc!{
                "$arrayElemAt": [
                "$total_users.total",
                0,
                ],
            },
            "rank": doc!{
                "$arrayElemAt": [
                "$user_rank.rank",
                0,
                ],
            },
        },
        },
    ];

    return match collection.aggregate(user_rank_pipeline, None).await {
        Ok(mut cursor) => {
            let mut data = Document::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                match result.get("rank") {
                    Some(rank) => {
                        data.insert("user_rank", rank);
                    }
                    None => {
                        data.insert("user_rank", 1);
                        return data;
                    }
                }

                match result.get("total_users") {
                    Some(total_users) => {
                        data.insert("total_users", total_users);
                    }
                    None => {
                        data.insert("total_users", 0);
                        return data;
                    }
                }
            }
            data
        }
        Err(_err) => {
            let mut data = Document::new();
            data.insert("user_rank", 1);
            data.insert("total_users", 0);
            data
        }
    };
}

pub fn get_default_range(rank: i64, page_size: i64, total_users: i64) -> i64 {
    let lower_range: i64;

    // if rank is in top half of the first page then return default range
    if rank <= page_size / 2 {
        lower_range = 1;
    }

    // if rank is in bottom half of the last page then return default range
    else if rank >= (total_users - page_size / 2) {
        lower_range = total_users - page_size;
    }

    // if rank is in middle then return modified range where rank will be placed in middle of page
    else {
        lower_range = rank - (page_size / 2 - 1);
    }
    return lower_range;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_range() {
        assert_eq!((9, 18), get_default_range(13, 10, 46));
    }

    #[test]
    fn fetch_normal_range() {
        assert_eq!((11, 20), get_default_range(15, 10, 46));
    }

    #[test]
    fn fetch_top_extreme_range() {
        assert_eq!((1, 10), get_default_range(3, 10, 46));
    }

    #[test]
    fn fetch_bottom_extreme_range() {
        assert_eq!((36, 46), get_default_range(43, 10, 46));
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetCompletedQuestsQuery {
    /*
    user address
     */
    addr: String,

    /*
    number of elements to show per page
     */
    page_size: i64,

    /*
    move forward or backward in the leaderboard
    */
    shift: i64,

    /*
    start of the timestamp range
    -> How many days back you want to start the leaderboard
     */
    start_timestamp: i64,

    /*
    end of the timestamp range
    -> When do you want to end it (ideally the moment the frontend makes the request till that timestamp)
    */
    end_timestamp: i64,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    let start_timestamp = query.start_timestamp;
    let end_timestamp = query.end_timestamp;

    if start_timestamp > end_timestamp {
        return get_error("Error querying ranks".to_string());
    }

    // get collection
    let users_collection = state.db.collection::<Document>("user_exp");

    // get params from query
    let address = query.addr.to_string();
    let page_size = query.page_size;
    let shift = query.shift;

    // get user rank and total users
    let stats = get_user_rank(&users_collection, &address, &start_timestamp, &end_timestamp).await;
    let total_users = stats.get("total_users").unwrap().as_i32().unwrap() as i64;
    let user_rank = stats.get("user_rank").unwrap().as_i32().unwrap() as i64;

    if total_users == 0 {
        return get_error("Error querying ranks".to_string());
    }

    let lower_range: i64;

    // get user position and get range to get page for showing user position
    if shift == 0 {
        lower_range = get_default_range(user_rank, page_size, total_users);
    }

    // get user position and set range if shift
    else {
        let default_lower_range = get_default_range(user_rank, page_size, total_users);

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
    }

    let paginated_leaderboard_pipeline = [
        doc! {
            "$match": doc!{
                "timestamp": doc!{
                    "$gte": start_timestamp,
                    "$lte": end_timestamp
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
            "$sort":doc! {
                "xp":-1,
                "timestamp":1,
                "achievements":-1,
                "address":1,
            }
        },
        doc! {
            "$skip": lower_range-1
        },
        doc! {
          "$limit":page_size
        },
        doc! {
            "$project": doc!{
                "xp": 1,
                "achievements": 1,
                "address":1,
            }
        }
    ];

    match users_collection.aggregate(paginated_leaderboard_pipeline, None).await {
        Ok(mut cursor) => {
            let mut res = Document::new();
            let mut ranking = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                ranking.push(result);
            }
            res.insert("ranking".to_string(), ranking);
            res.insert("first_elt_position".to_string(), if lower_range == 0 { 1 } else { lower_range });
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(_err) => {
            get_error("Error querying ranks".to_string())
        }
    }
}
