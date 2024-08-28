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
use axum_auto_routes::route;

use crate::utils::get_timestamp_from_days;
use axum::http::{header, Response};
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn get_user_rank(
    collection: &Collection<Document>,
    address: &String,
    timestamp: &i64,
) -> Document {
    let user_rank_pipeline = vec![
        doc! {
            "$match": doc! {
                "timestamp": doc! {
                    "$gte": timestamp,
                }
            }
        },
        doc! {
            "$sort": doc! {
                "experience": -1,
                "timestamp": 1,
                "_id": 1
            }
        },
        doc! {
            "$addFields": doc! {
                "tempSortField": 1
            }
        },
        doc! {
            "$setWindowFields": doc! {
                "sortBy": doc! {
                    "tempSortField": -1
                },
                "output": doc! {
                    "rank": doc! {
                        "$documentNumber": doc! {}
                    }
                }
            }
        },
        doc! {
            "$facet": doc! {
                "total_users": [
                    doc! {
                        "$count": "total"
                    }
                ],
                "user_rank": [
                    doc! {
                        "$match": doc! {
                            "_id": address
                        }
                    },
                    doc! {
                        "$project": doc! {
                            "_id": 0,
                            "rank": "$rank"
                        }
                    }
                ]
            }
        },
        doc! {
            "$project": doc! {
                "total_users": doc! {
                    "$arrayElemAt": [
                        "$total_users.total",
                        0
                    ]
                },
                "rank": doc! {
                    "$arrayElemAt": [
                        "$user_rank",
                        0
                    ]
                }
            }
        },
        doc! {
            "$project": doc! {
                "total_users": 1,
                "rank": "$rank.rank"
            }
        },
    ];

    // add allow disk use to view options
    let view_options = mongodb::options::AggregateOptions::builder()
        .allow_disk_use(true)
        .build();

    return match collection.aggregate(user_rank_pipeline, view_options).await {
        Ok(mut cursor) => {
            let mut data = Document::new();

            while let Some(result) = cursor.try_next().await.unwrap() {
                match result.get("rank") {
                    Some(rank) => {
                        data.insert("user_rank", rank);
                    }
                    None => {
                        data.insert("user_rank", 1);
                    }
                }

                match result.get("total_users") {
                    Some(total_users) => {
                        data.insert("total_users", total_users);
                    }
                    None => {
                        data.insert("total_users", 0);
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
        assert_eq!((9), get_default_range(13, 10, 46));
    }

    #[test]
    fn fetch_normal_range() {
        assert_eq!((11), get_default_range(15, 10, 46));
    }

    #[test]
    fn fetch_top_extreme_range() {
        assert_eq!((1), get_default_range(3, 10, 46));
    }

    #[test]
    fn fetch_bottom_extreme_range() {
        assert_eq!((36), get_default_range(43, 10, 46));
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

    duration: String,
}

#[route(get, "/leaderboard/get_ranking")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetCompletedQuestsQuery>,
) -> impl IntoResponse {
    // check value of duration and set time_gap accordingly using match and respective timestamp
    let time_gap = match query.duration.as_str() {
        "week" => get_timestamp_from_days(7),
        "month" => get_timestamp_from_days(30),
        "all" => 0,
        _ => {
            return get_error("Invalid duration".to_string());
        }
    };
    // get collection
    let users_collection = state.db.collection::<Document>("leaderboard_table");

    // get params from query
    let address = query.addr.to_string();
    let page_size = query.page_size;
    let shift = query.shift;

    // get user rank and total users
    let stats = get_user_rank(&users_collection, &address, &time_gap).await;
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
                    "$gte": time_gap,
                }
            }
        },
        doc! {
            "$sort":doc! {
                "experience":-1,
                "timestamp":1,
                "_id":1,
            }
        },
        doc! {
            "$skip": lower_range-1
        },
        doc! {
          "$limit":page_size
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
    ];

    match users_collection
        .aggregate(paginated_leaderboard_pipeline, None)
        .await
    {
        Ok(mut cursor) => {
            let mut res = Document::new();
            let mut ranking = Vec::new();
            while let Some(result) = cursor.try_next().await.unwrap() {
                ranking.push(result);
            }
            res.insert("ranking".to_string(), ranking);
            res.insert(
                "first_elt_position".to_string(),
                if lower_range == 0 { 1 } else { lower_range },
            );

            // Set caching response
            let expires = Utc::now() + chrono::Duration::minutes(5);
            let caching_response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CACHE_CONTROL, "public, max-age=300")
                .header(header::EXPIRES, expires.to_rfc2822())
                .body(Json(res).to_string());

            return caching_response.unwrap().into_response();
        }
        Err(_err) => get_error("Error querying ranks".to_string()),
    }
}
