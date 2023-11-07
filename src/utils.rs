use futures::TryStreamExt;
use crate::models::{AchievementDocument, AppState, CompletedTasks, TaskDocument, QuestDocument};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Response as HttpResponse, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use mongodb::{bson::doc, options::UpdateOptions, results::UpdateResult, Collection};
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
            Some(id) => {
                // lookup from the tasks collection and get quest id
                let tasks_collection: Collection<TaskDocument> = self.db.collection("tasks");
                let filter = doc! { "id": task_id };
                let mut quest_id = 0;
                let mut cursor = tasks_collection.find(filter, None).await?;
                while let Some(doc) = cursor.try_next().await? {
                    quest_id = doc.quest_id;
                }

                // get total tasks for a specific quest id from task collection
                let filter = doc! { "quest_id": quest_id };
                let mut total_tasks = Vec::new();
                let mut cursor = tasks_collection.find(filter, None).await?;
                while let Some(doc) = cursor.try_next().await? {
                    total_tasks.push(doc.id);
                }

                // get total experience for a specific quest id from quest collection
                let quests_collection: Collection<QuestDocument> = self.db.collection("quests");
                let filter = doc! { "id": quest_id };
                let mut experience: i32 = 0;
                let mut cursor = quests_collection.find(filter, None).await?;
                while let Some(doc) = cursor.try_next().await? {
                    experience = doc.experience as i32;
                }


                // flag value to check if quest completed (initially we assume it is and then check if any task is not completed)
                let mut result = true;

                // get completed tasks for a specific quest id from completed_tasks collection
                for &item in total_tasks.iter() {
                    let filter = doc! { "address": addr.to_string(),"task_id": item };
                    match completed_tasks_collection.find(filter, None).await {
                        Ok(mut cursor) => {
                            if cursor.try_next().await?.is_none() {
                                result = false;
                                break;
                            }
                        }
                        Err(e) => {
                            result = false
                        }
                    }
                }

                // save the user_exp document in the collection if the quest is completed
                if result == true {
                    // save the user_exp document in the collection
                    let user_exp_collection = self.db.collection("user_exp");
                    // add doc with address ,experience and timestamp
                    let timestamp: f64 = Utc::now().timestamp_millis() as f64;
                    let document = doc! { "address": addr.to_string(), "experience":experience, "timestamp":timestamp};
                    user_exp_collection.insert_one(document, None).await?;
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
            Some(id) => {
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
                let yay = user_exp_collection.insert_one(document, None).await?;
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
