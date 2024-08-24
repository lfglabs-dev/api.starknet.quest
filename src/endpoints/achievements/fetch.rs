use std::sync::Arc;

use crate::{
    models::{AchievementCategoryDocument, AchievementQuery, AppState, UserAchievements},
    utils::get_error,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document};
use starknet::core::types::FieldElement;

#[route(
  get,
  "/achievements/fetch"
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AchievementQuery>,
) -> impl IntoResponse {
    let addr = FieldElement::to_string(&query.addr);
    let achievement_categories = state
        .db
        .collection::<AchievementCategoryDocument>("achievement_categories");
    let pipeline = vec![
        doc! {
          "$lookup": {
            "from": "achievements",
            "localField": "id",
            "foreignField": "category_id",
            "as": "achievement"
          }
        },
        doc! {"$unwind": "$achievement" },
        doc! {
          "$lookup": {
            "from": "achieved",
            "let": { "achievement_id": "$achievement.id" },
            "pipeline": [
                { "$match": {
                  "$expr": {
                    "$and": [
                      { "$eq": ["$achievement_id", "$$achievement_id"] },
                      { "$eq": ["$addr", addr] }
                    ]
                  }
                } }
              ],
              "as": "achieved"
          }
        },
        doc! {
          "$project": {
            "_id": 0,
            "category_id": "$id",
            "category_name": "$name",
            "category_desc": "$desc",
            "category_img_url": "$img_url",
            "category_type": "$type",
            "category_disabled": "$disabled",
            "category_override_verified_type": "$override_verified_type",
            "achievements": {
              "id": "$achievement.id",
              "name": "$achievement.name",
              "short_desc": "$achievement.short_desc",
              "title": {
                "$cond": [
                  { "$eq": [{ "$size": "$achieved" }, 0] },
                  "$achievement.todo_title",
                  "$achievement.done_title"
                ]
              },
              "desc": {
                "$cond": [
                  { "$eq": [{ "$size": "$achieved" }, 0] },
                  "$achievement.todo_desc",
                  "$achievement.done_desc"
                ]
              },
              "completed": { "$ne": [{ "$size": "$achieved" }, 0] },
              "verify_type": "$achievement.verify_type",
              "img_url": "$achievement.img_url"
            }
          }
        },
        doc! {
          "$group": {
            "_id": {
              "category_id": "$category_id",
              "category_name": "$category_name",
              "category_desc": "$category_desc",
              "category_img_url": "$category_img_url",
              "category_type": "$category_type",
              "category_disabled": "$category_disabled",
              "category_override_verified_type": "$category_override_verified_type",
            },
            "achievements": { "$push": "$achievements" }
          }
        },
        doc! {
          "$project": {
            "category_id": "$_id.category_id",
            "category_name": "$_id.category_name",
            "category_desc": "$_id.category_desc",
            "category_img_url": "$_id.category_img_url",
            "category_type": "$_id.category_type",
            "category_disabled": "$_id.category_disabled",
            "category_override_verified_type": "$_id.category_override_verified_type",
            "achievements": 1,
            "_id": 0
          }
        },
        doc! {
            "$sort": { "category_id": 1 }
        },
    ];

    match achievement_categories.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut achievements: Vec<UserAchievements> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(achievement) = from_document::<UserAchievements>(document) {
                            if !achievement.category_disabled {
                                achievements.push(achievement);
                            }
                        }
                    }
                    _ => continue,
                }
            }
            (StatusCode::OK, Json(achievements)).into_response()
        }
        Err(e) => get_error(format!("Error fetching user achievements: {}", e)),
    }
}
