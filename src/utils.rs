use futures::TryStreamExt;
use crate::models::{AchievementDocument, AppState, CompletedTasks, LeaderboardTable, UserExperience};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Response as HttpResponse, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use mongodb::{bson::doc, options::UpdateOptions, results::UpdateResult, Collection, Database, Cursor, IndexModel};
use starknet::signers::Signer;
use starknet::{
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::LocalWallet,
};
use std::fmt::Write;
use std::result::Result;
use std::str::FromStr;
use chrono::{Utc};
#[macro_export]
macro_rules! pub_struct {
    ($($derive:path),*; $name:ident {$($field:ident: $t:ty),* $(,)?}) => {
        #[derive($($derive),*)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub async fn get_nft(
    quest_id: u32,
    task_id: u32,
    addr: &FieldElement,
    nft_level: u32,
    signer: &LocalWallet,
) -> Result<(u64, Signature), Box<dyn std::error::Error + Send + Sync>> {
    let token_id = nft_level as u64 + 100 * (rand::random::<u64>() % (2u64.pow(32)));
    let hashed = pedersen_hash(
        &pedersen_hash(
            &pedersen_hash(
                &pedersen_hash(&FieldElement::from(token_id), &FieldElement::ZERO),
                &FieldElement::from(quest_id),
            ),
            &FieldElement::from(task_id),
        ),
        addr,
    );
    let sig = signer.sign_hash(&hashed).await?;
    Ok((token_id, sig))
}

pub fn get_error(error: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
}

pub fn get_error_redirect(redirect_uri: String, error: String) -> Response {
    let err_msg_encoded =
        percent_encoding::utf8_percent_encode(&error, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let redirect_url = format!("{}&error_msg={}", redirect_uri, err_msg_encoded);
    let uri = match Uri::from_str(&redirect_url) {
        Ok(uri) => uri,
        Err(_) => return get_error("Failed to create URI from redirect URL".to_string()),
    };

    let response = match HttpResponse::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::from("Redirecting..."))
    {
        Ok(response) => response,
        Err(_) => return get_error("Failed to create HTTP response".to_string()),
    };

    response.into_response()
}

pub fn success_redirect(redirect_uri: String) -> Response {
    let uri = match Uri::from_str(&redirect_uri) {
        Ok(uri) => uri,
        Err(_) => return get_error("Failed to create URI from redirect URL".to_string()),
    };

    let response = match HttpResponse::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::from("Redirecting..."))
    {
        Ok(response) => response,
        Err(_) => return get_error("Failed to create HTTP response".to_string()),
    };

    response.into_response()
}

#[async_trait]
pub trait CompletedTasksTrait {
    async fn upsert_completed_task(
        &self,
        addr: FieldElement,
        task_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error>;
}

#[async_trait]
impl CompletedTasksTrait for AppState {
    async fn upsert_completed_task(
        &self,
        addr: FieldElement,
        task_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let completed_tasks_collection: Collection<CompletedTasks> =
            self.db.collection("completed_tasks");
        let filter = doc! { "address": addr.to_string(), "task_id": task_id };
        let update = doc! { "$setOnInsert": { "address": addr.to_string(), "task_id": task_id } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = completed_tasks_collection
            .update_one(filter, update, options)
            .await?;

        match &result.upserted_id {
            Some(_id) => {
                let pipeline = vec![
                    doc! {
                        "$match": doc!{
                        "address": addr.to_string(),
                    },
                    },
                    doc! {
                        "$lookup": doc! {
                        "from": "tasks",
                        "localField": "task_id",
                        "foreignField": "id",
                        "as": "associatedTask",
                    },
                    },
                    doc! {
                        "$unwind": "$associatedTask",
                    },
                    doc! {
                       "$project": doc! {
                        "address": "$address",
                        "task_id": "$task_id",
                       "quest_id": "$associatedTask.quest_id",
                    },
                    },
                    doc! {
                        "$group": doc! {
                        "_id": "$quest_id",
                        "done": doc! {
                            "$sum": 1,
                        },
                    },
                    },
                    doc! {
                        "$lookup": doc! {
                        "from": "tasks",
                        "localField": "_id",
                        "foreignField": "quest_id",
                        "as": "tasks",
                    },
                    },
                    doc! {
                        "$match": doc! {
                        "$expr": {
                            "$eq": [
                            "$done",
                            {
                                "$size": "$tasks",
                            },
                            ],
                        },
                    },
                    },
                    doc! {
                        "$match": doc! {
                        "tasks": doc! {
                            "$elemMatch": {
                                "id": task_id,
                            },
                        },
                    },
                    },
                    doc! {
                        "$lookup": doc! {
                        "from": "quests",
                        "localField": "_id",
                        "foreignField": "id",
                        "as": "associatedQuests",
                    }
                    },
                    doc! {
                        "$unwind": "$associatedQuests",
                    },
                    doc! {
                        "$project": doc! {
                            "_id": 0,
                            "experience": "$associatedQuests.experience",
                        }
                    },
                ];
                match completed_tasks_collection.aggregate(pipeline, None).await {
                    Ok(mut cursor) => {
                        let mut experience = 0;
                        while let Some(response) = cursor.try_next().await.unwrap() {
                            experience = response.get("experience").unwrap().as_i32().unwrap();
                        }

                        // return result if experience is 0 (quest is not completed)
                        if experience == 0 {
                            return Ok(result);
                        }

                        // save the user_exp document in the collection
                        let user_exp_collection = self.db.collection("user_exp");
                        // add doc with address ,experience and timestamp
                        let timestamp: f64 = Utc::now().timestamp_millis() as f64;
                        let document = doc! { "address": addr.to_string(), "experience":experience, "timestamp":timestamp};
                        user_exp_collection.insert_one(document, None).await?;
                        let view_collection: Collection<LeaderboardTable> = self.db.collection("leaderboard_table");
                        update_leaderboard(view_collection, addr.to_string(), experience.into(), timestamp).await;
                    }
                    Err(_e) => {
                        get_error("Error querying quests".to_string());
                    }
                }
            }
            None => {}
        }

        Ok(result)
    }
}

pub fn to_hex(felt: FieldElement) -> String {
    let bytes = felt.to_bytes_be();
    let mut result = String::with_capacity(bytes.len() * 2 + 2);
    result.push_str("0x");
    for byte in bytes {
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}

#[async_trait]
pub trait AchievementsTrait {
    async fn upsert_completed_achievement(
        &self,
        addr: FieldElement,
        achievement_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error>;

    async fn get_achievement(
        &self,
        achievement_id: u32,
    ) -> Result<Option<AchievementDocument>, mongodb::error::Error>;
}

#[async_trait]
impl AchievementsTrait for AppState {
    async fn upsert_completed_achievement(
        &self,
        addr: FieldElement,
        achievement_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let achieved_collection: Collection<CompletedTasks> = self.db.collection("achieved");
        let filter = doc! { "addr": addr.to_string(), "achievement_id": achievement_id };
        let update =
            doc! { "$setOnInsert": { "addr": addr.to_string(), "achievement_id": achievement_id } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = achieved_collection
            .update_one(filter, update, options)
            .await?;


        match &result.upserted_id {
            Some(_id) => {
                // Check if the document was modified
                let achievement_collection: Collection<AchievementDocument> = self.db.collection("achievements");
                // Define a query using the `doc!` macro.
                let query = doc! { "id": achievement_id };
                let mut experience: i32 = 0;

                let mut cursor = achievement_collection.find(query, None).await?;
                // Iterate over the results.
                while let Some(doc) = cursor.try_next().await? {
                    experience = doc.experience as i32;
                }

                let user_exp_collection = self.db.collection("user_exp");
                // add doc with address ,experience and timestamp
                let timestamp: f64 = Utc::now().timestamp_millis() as f64;
                let document = doc! { "address": addr.to_string(), "experience":experience, "timestamp":timestamp};
                user_exp_collection.insert_one(document, None).await?;
                let view_collection: Collection<LeaderboardTable> = self.db.collection("leaderboard_table");
                update_leaderboard(view_collection, addr.to_string(), experience.into(), timestamp).await;
            }
            None => {}
        }
        Ok(result)
    }

    async fn get_achievement(
        &self,
        achievement_id: u32,
    ) -> Result<Option<AchievementDocument>, mongodb::error::Error> {
        let achievements_collection: Collection<AchievementDocument> =
            self.db.collection("achievements");
        let query = doc! {
            "id": achievement_id
        };
        let result = achievements_collection.find_one(query, None).await;
        result
    }
}

#[async_trait]
pub trait DeployedTimesTrait {
    async fn upsert_deployed_timestamp(
        &self,
        addr: FieldElement,
        timestamp: u32,
    ) -> Result<UpdateResult, mongodb::error::Error>;
}

#[async_trait]
impl DeployedTimesTrait for AppState {
    async fn upsert_deployed_timestamp(
        &self,
        addr: FieldElement,
        timestamp: u32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let deployed_times_collection: Collection<CompletedTasks> =
            self.db.collection("deployed_times");
        let filter = doc! { "addr": addr.to_string() };
        let update = doc! { "$setOnInsert": { "addr": to_hex(addr), "timestamp": timestamp } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = deployed_times_collection
            .update_one(filter, update, options)
            .await;
        result
    }
}

pub async fn update_leaderboard(view_collection: Collection<LeaderboardTable>, address: String, experience: i64, timestamp: f64) {
    // get current experience and new experience to it
    let mut old_experience = 0;
    let filter = doc! { "_id": &*address };
    let mut cursor: Cursor<LeaderboardTable> = view_collection.find(filter, None).await.unwrap();
    while let Some(doc) = cursor.try_next().await.unwrap() {
        old_experience = doc.experience;
    }


    // update the view collection
    let filter = doc! { "_id": &*address };
    let update = doc! { "$set": { "experience": old_experience + experience, "timestamp": timestamp } };
    let options = UpdateOptions::builder().upsert(true).build();
    view_collection.update_one(filter, update, options).await.unwrap();
}


pub async fn add_leaderboard_watcher(db: &Database) {
    let view_collection_name = "leaderboard_table";

    let pipeline = vec![
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
        doc! { "$merge" : doc! { "into":  view_collection_name , "on": "_id",  "whenMatched": "replace", "whenNotMatched": "insert" } },
    ];

    let view_collection: Collection<LeaderboardTable> = db.collection::<LeaderboardTable>(view_collection_name);
    let source_collection = db.collection::<UserExperience>("user_exp");

    // create materialised view
    source_collection.aggregate(pipeline, None).await.unwrap();

    let index = IndexModel::builder()
        .keys(doc! { "experience": -1})
        .build();

    //add indexing to materialised view
    view_collection.create_index(index, None).await.unwrap();
}