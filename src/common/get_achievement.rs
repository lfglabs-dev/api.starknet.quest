use std::sync::Arc;

use crate::models::AppState;
use crate::models::{AchievementCategoryDocument, UserAchievementsCategory};
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use starknet::core::types::FieldElement;

pub async fn get_achievement(
    state: &Arc<AppState>,
    addr: &FieldElement,
    category_id: u32,
) -> Result<UserAchievementsCategory, String> {
    let achievement_categories = state
        .db
        .collection::<AchievementCategoryDocument>("achievement_categories");
    let pipeline = vec![
        doc! {
            "$match": {
                "id": category_id,
            }
        },
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
                      { "$eq": ["$addr", FieldElement::to_string(&addr)] }
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
            "achievements": {
              "id": "$achievement.id",
              "completed": { "$ne": [{ "$size": "$achieved" }, 0] },
              "verify_type": "$achievement.verify_type",
            }
          }
        },
        doc! {
          "$group": {
            "_id": {
              "category_id": "$category_id",
            },
            "achievements": { "$push": "$achievements" }
          }
        },
        doc! {
          "$project": {
            "category_id": "$_id.category_id",
            "achievements": 1,
            "_id": 0
          }
        },
    ];

    match achievement_categories.aggregate(pipeline, None).await {
        Ok(mut cursor) => match cursor.next().await {
            Some(Ok(document)) => match from_document::<UserAchievementsCategory>(document) {
                Ok(achievement_category) => Ok(achievement_category),
                Err(e) => Err(format!("Error deserializing document : {}", e)),
            },
            Some(Err(e)) => Err(format!("No data found: {}", e)),
            None => Err("No data found".to_string()),
        },
        Err(e) => Err(format!("Error fetching user achievements: {}", e)),
    }
}
