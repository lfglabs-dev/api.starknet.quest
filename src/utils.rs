use crate::logger::Logger;
use crate::models::{
    AchievementDocument, AppState, BoostTable, CompletedTasks, LeaderboardTable, QuestDocument,
    QuestTaskDocument, UserExperience,
};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Response as HttpResponse, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use chrono::{Duration as dur, Utc};
use futures::TryStreamExt;
use mongodb::{
    bson::doc, options::UpdateOptions, results::UpdateResult, Collection, Cursor, Database,
    IndexModel,
};
use rand::distributions::{Distribution, Uniform};
use serde_json::json;
use starknet::signers::Signer;
use starknet::{
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::LocalWallet,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::result::Result;
use std::str::FromStr;
use std::{fmt::Write, sync::Arc};
use tokio::time::{sleep, Duration};

#[macro_export]
macro_rules! pub_struct {
    ($($derive:path),*; $name:ident {$($field:ident: $t:ty),* $(,)?}) => {
        #[derive($($derive),*)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}
   
macro_rules! check_authorization {
    ($headers:expr,$secret_key:expr) => {
        match $headers.get("Authorization") {
            Some(auth_header) => {
                let validation = Validation::new(Algorithm::HS256);
                let token = auth_header
                    .to_str()
                    .unwrap()
                    .to_string()
                    .split(" ")
                    .collect::<Vec<&str>>()[1]
                    .to_string();

                match decode::<JWTClaims>(
                    &token,
                    &DecodingKey::from_secret($secret_key),
                    &validation,
                ) {
                    Ok(token_data) => token_data.claims.sub,
                    Err(_e) => {
                        return get_error("Invalid token".to_string());
                    }
                }
            }
            None => return get_error("missing auth header".to_string()),
        }
    };
}

pub async fn get_nft(
    quest_id: u32,
    task_id: u32,
    addr: &FieldElement,
    nft_level: u32,
    signer: &LocalWallet,
) -> Result<(u64, Signature), Box<dyn std::error::Error + Send + Sync>> {
    let token_id = match nft_level < 100 {
        true => nft_level as u64 + 100 * (rand::random::<u64>() % (2u64.pow(32))),
        false => (rand::random::<u64>() + nft_level as u64 * 0x2000000) * 100 + 99,
    };
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

pub fn calculate_hash(t: &String) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
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
        let created_at = Utc::now().timestamp_millis();
        let filter = doc! { "address": addr.to_string(), "task_id": task_id };
        let update = doc! { "$setOnInsert": { "address": addr.to_string(), "task_id": task_id , "timestamp":created_at} };

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
                        let view_collection: Collection<LeaderboardTable> =
                            self.db.collection("leaderboard_table");
                        update_leaderboard(
                            view_collection,
                            addr.to_string(),
                            experience.into(),
                            timestamp,
                        )
                        .await;
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

    async fn upsert_claimed_achievement(
        &self,
        addr: String,
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
        let created_at = Utc::now().timestamp_millis();
        let filter = doc! { "addr": addr.to_string(), "achievement_id": achievement_id };
        let update = doc! { "$setOnInsert": { "addr": addr.to_string(), "achievement_id": achievement_id , "timestamp":created_at } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = achieved_collection
            .update_one(filter, update, options)
            .await?;

        match &result.upserted_id {
            Some(_id) => {
                // Check if the document was modified
                let achievement_collection: Collection<AchievementDocument> =
                    self.db.collection("achievements");
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
                let view_collection: Collection<LeaderboardTable> =
                    self.db.collection("leaderboard_table");
                update_leaderboard(
                    view_collection,
                    addr.to_string(),
                    experience.into(),
                    timestamp,
                )
                .await;
            }
            None => {}
        }
        Ok(result)
    }

    async fn upsert_claimed_achievement(
        &self,
        addr: String,
        achievement_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let claimed_achievements_collection: Collection<CompletedTasks> =
            self.db.collection("claimed_achievements");
        let filter = doc! { "address": addr.to_string(), "id": achievement_id };
        let update = doc! { "$setOnInsert": { "address": addr.to_string(), "id": achievement_id } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = claimed_achievements_collection
            .update_one(filter, update, options)
            .await;
        result
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

pub fn get_timestamp_from_days(days: i64) -> i64 {
    // take input as week , month and all time and return the timestamp range
    let time_gap = if days > 0 {
        (Utc::now() - dur::days(days)).timestamp_millis()
    } else {
        0
    };
    time_gap
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

pub async fn fetch_json_from_url(url: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Failed to get JSON response: {}", e)),
        },
        Err(e) => Err(format!("Failed to send request: {}", e)),
    }
}

pub async fn update_leaderboard(
    view_collection: Collection<LeaderboardTable>,
    address: String,
    experience: i64,
    timestamp: f64,
) {
    // get current experience and new experience to it
    let mut old_experience = 0;
    let filter = doc! { "_id": &*address };
    let mut cursor: Cursor<LeaderboardTable> = view_collection.find(filter, None).await.unwrap();
    while let Some(doc) = cursor.try_next().await.unwrap() {
        old_experience = doc.experience;
    }

    // update the view collection
    let filter = doc! { "_id": &*address };
    let update =
        doc! { "$set": { "experience": old_experience + experience, "timestamp": timestamp } };
    let options = UpdateOptions::builder().upsert(true).build();
    view_collection
        .update_one(filter, update, options)
        .await
        .unwrap();
}

pub async fn add_leaderboard_table(db: &Database) {
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

    let view_collection: Collection<LeaderboardTable> =
        db.collection::<LeaderboardTable>(view_collection_name);
    let source_collection = db.collection::<UserExperience>("user_exp");

    // create materialised view
    source_collection.aggregate(pipeline, None).await.unwrap();

    //create multiple indexes to speed it up
    let timestamp_only = IndexModel::builder().keys(doc! { "timestamp":1}).build();
    view_collection
        .create_index(timestamp_only, None)
        .await
        .unwrap();
    let addrs_only = IndexModel::builder().keys(doc! { "_id":1}).build();
    view_collection
        .create_index(addrs_only, None)
        .await
        .unwrap();
    let compound_index = IndexModel::builder()
        .keys(doc! { "experience": -1,"timestamp":1,"_id":1})
        .build();
    view_collection
        .create_index(compound_index, None)
        .await
        .unwrap();
}

pub async fn fetch_and_update_boosts_winner(
    boost_collection: Collection<BoostTable>,
    completed_tasks_collection: Collection<CompletedTasks>,
    interval: u64,
    logger: Logger,
) {
    loop {
        let pipeline = vec![doc! {
            "$match": {
                "expiry":{
                    "$lt": Utc::now().timestamp_millis()
                },
                "winner": {
                    "$eq": null,
                },
            }
        }];
        match boost_collection.aggregate(pipeline, None).await {
            Ok(mut cursor) => {
                while let Some(doc) = cursor.try_next().await.unwrap() {
                    let mut num_of_winners = doc.get("num_of_winners").unwrap().as_i32().unwrap();
                    // use this variable to add some extra winners so that we have some extra winners incase anyone user repeats
                    let extra_winners = 10;
                    match doc.get("quests") {
                        Some(quests_res) => {
                            let quests = quests_res.as_array().unwrap();
                            let mut address_list: Vec<FieldElement> = Vec::new();
                            for quest in quests {
                                let get_users_per_quest_pipeline = vec![
                                    doc! {
                                        "$lookup": doc! {
                                            "from": "tasks",
                                            "localField": "task_id",
                                            "foreignField": "id",
                                            "as": "associated_tasks"
                                        }
                                    },
                                    doc! {
                                        "$match": doc! {
                                            "$expr": doc! {
                                                "$eq": [
                                                    doc! {
                                                        "$first": "$associated_tasks.quest_id"
                                                    },
                                                    quest
                                                ]
                                            }
                                        }
                                    },
                                    doc! {
                                        "$group": doc! {
                                            "_id": "$address",
                                            "tasks_list": doc! {
                                                "$push": doc! {
                                                    "$arrayElemAt": [
                                                        "$associated_tasks",
                                                        0
                                                    ]
                                                }
                                            }
                                        }
                                    },
                                    doc! {
                                        "$unwind": "$tasks_list"
                                    },
                                    doc! {
                                        "$group": doc! {
                                            "_id": doc! {
                                                "address": "$_id",
                                                "quest_id": "$tasks_list.quest_id"
                                            },
                                            "tasks_array": doc! {
                                                "$push": "$tasks_list"
                                            }
                                        }
                                    },
                                    doc! {
                                        "$project": doc! {
                                            "_id": 0,
                                            "address": "$_id.address",
                                            "quest_id": "$_id.quest_id",
                                            "tasks_array": 1
                                        }
                                    },
                                    doc! {
                                        "$lookup": doc! {
                                            "from": "tasks",
                                            "localField": "quest_id",
                                            "foreignField": "quest_id",
                                            "as": "associatedTasks"
                                        }
                                    },
                                    doc! {
                                        "$match": doc! {
                                            "$expr": doc! {
                                                "$eq": [
                                                    doc! {
                                                        "$size": "$tasks_array"
                                                    },
                                                    doc! {
                                                        "$size": "$associatedTasks"
                                                    }
                                                ]
                                            }
                                        }
                                    },
                                    doc! {
                                        "$project": doc! {
                                            "address": "$address"
                                        }
                                    },
                                    doc! {
                                        "$sample":{
                                            "size":num_of_winners+extra_winners
                                        }
                                    },
                                ];
                                match completed_tasks_collection
                                    .aggregate(get_users_per_quest_pipeline, None)
                                    .await
                                {
                                    Ok(mut cursor) => {
                                        while let Some(doc) = cursor.try_next().await.unwrap() {
                                            let address =
                                                doc.get("address").unwrap().as_str().unwrap();
                                            let formatted_address =
                                                FieldElement::from_str(address).unwrap();
                                            address_list.push(formatted_address);
                                        }
                                    }
                                    Err(_err) => {}
                                }
                            }

                            // skip if no user has completed quests
                            if address_list.len() == 0 {
                                continue;
                            }
                            let mut random_index;
                            let mut winner_array: Vec<String> = Vec::new();

                            // if length of address list is 1 then select the only user
                            if address_list.len() == 1 {
                                let winner = &address_list[0].to_string();
                                let formatted_winner = FieldElement::from_str(winner).unwrap();
                                winner_array.push(to_hex(formatted_winner));
                            }
                            // else select random users
                            else {
                                let mut current_winner_index = 0;
                                // handle case when number of winners is greater than number of users then assign all users as winners
                                if address_list.len() < num_of_winners as usize {
                                    num_of_winners = address_list.len() as i32;
                                }
                                let mut iter_index = 0;
                                loop {
                                    let mut rng = rand::thread_rng();

                                    let die = Uniform::new(0, address_list.len());
                                    random_index = die.sample(&mut rng);

                                    let winner = &address_list[random_index].to_string();
                                    let formatted_winner = FieldElement::from_str(winner).unwrap();
                                    if !winner_array.contains(&to_hex(formatted_winner)) {
                                        winner_array.push(to_hex(formatted_winner));
                                        current_winner_index += 1;
                                    }
                                    iter_index += 1;
                                    if current_winner_index == (num_of_winners) as usize
                                        || iter_index == address_list.len()
                                    {
                                        break;
                                    }
                                }
                            }

                            let filter = doc! { "id": doc.get("id").unwrap().as_i32().unwrap() };
                            let update = doc! { "$set": { "winner": winner_array  } };
                            let options = UpdateOptions::builder().upsert(true).build();
                            boost_collection
                                .update_one(filter, update, options)
                                .await
                                .unwrap();
                        }
                        None => {
                            logger.info("No winners found");
                        }
                    }
                }
            }
            Err(_err) => logger.info(_err.to_string()),
        };

        sleep(Duration::from_secs(interval)).await;
    }
}

pub fn run_boosts_raffle(db: &Database, interval: u64, logger: Logger) {
    let boost_collection = db.collection::<BoostTable>("boosts");
    let completed_tasks_collection = db.collection::<CompletedTasks>("completed_tasks");
    tokio::spawn(fetch_and_update_boosts_winner(
        boost_collection,
        completed_tasks_collection,
        interval,
        logger,
    ));
}

pub async fn verify_task_auth(
    user: String,
    task_collection: &Collection<QuestTaskDocument>,
    id: &i32,
) -> bool {
    if user == "super_user" {
        return true;
    }

    let pipeline = vec![
        doc! {
            "$match": doc! {
                "id": id
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quests",
                "localField": "quest_id",
                "foreignField": "id",
                "as": "quest"
            }
        },
        doc! {
            "$project": doc! {
                "quest.issuer": 1
            }
        },
        doc! {
            "$unwind": doc! {
                "path": "$quest"
            }
        },
        doc! {
            "$project": doc! {
                "issuer": "$quest.issuer"
            }
        },
    ];
    let mut existing_quest = task_collection.aggregate(pipeline, None).await.unwrap();

    let mut issuer = String::new();
    while let Some(doc) = existing_quest.try_next().await.unwrap() {
        issuer = doc.get("issuer").unwrap().as_str().unwrap().to_string();
    }
    if issuer == user {
        return true;
    }
    false
}

pub async fn verify_quest_auth(
    user: String,
    quest_collection: &Collection<QuestDocument>,
    id: &i64,
) -> bool {
    if user == "super_user" {
        return true;
    }

    let filter = doc! { "id": id, "issuer": user };

    let existing_quest = quest_collection.find_one(filter, None).await.unwrap();

    match existing_quest {
        Some(_) => true,
        None => false,
    }
}
pub async fn make_api_request(endpoint: &str, addr: &str, api_key: Option<&str>) -> bool {
    let client = reqwest::Client::new();
    let request_builder = client.post(endpoint).json(&json!({
        "address": addr,
    }));
    let key = api_key.unwrap_or("");
    let request_builder = match key.is_empty() {
        true => request_builder,
        false => request_builder.header("apiKey", key),
    };
    match request_builder.send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => {
                //check value of result in json
                if let Some(data) = json.get("data") {
                    if let Some(res) = data.get("result") {
                        return res.as_bool().unwrap();
                    }
                }
                false
            }
            Err(_) => false,
        },
        Err(_) => false,
    };
    false
}

// required for axum_auto_routes
pub trait WithState: Send {
    fn to_router(self: Box<Self>, shared_state: Arc<AppState>) -> Router;

    fn box_clone(&self) -> Box<dyn WithState>;
}

impl WithState for Router<Arc<AppState>, Body> {
    fn to_router(self: Box<Self>, shared_state: Arc<AppState>) -> Router {
        self.with_state(shared_state)
    }

    fn box_clone(&self) -> Box<dyn WithState> {
        Box::new((*self).clone())
    }
}

impl Clone for Box<dyn WithState> {
    fn clone(&self) -> Box<dyn WithState> {
        self.box_clone()
    }
}