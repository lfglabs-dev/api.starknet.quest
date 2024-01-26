use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::{
    core::types::FieldElement,
    providers::{jsonrpc::HttpTransport, JsonRpcClient},
};

use crate::config::Config;

pub_struct!(;AppState {
    conf: Config,
    provider: JsonRpcClient<HttpTransport>,
    db: Database,
});

pub_struct!(Debug, Serialize, Deserialize; NFTItem {
    img: String,
    level: u32,
});

pub_struct!(Debug, Serialize, Deserialize; QuestDocument {
    id: u32,
    name: String,
    desc: String,
    additional_desc: Option<String>,
    issuer: String,
    category: String,
    rewards_endpoint: String,
    logo: String,
    rewards_img: String,
    rewards_title: String,
    rewards_description: Option<String>,
    rewards_nfts: Vec<NFTItem>,
    img_card: String,
    title_card: String,
    hidden: bool,
    disabled: bool,
    expiry: Option<bson::DateTime>,
    expiry_timestamp: Option<String>,
    mandatory_domain: Option<String>,
    expired: Option<bool>,
    experience: i64,
});

pub_struct!(Deserialize; CompletedTasks {
    address: String,
    task_id: u32,
});

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTaskDocument {
    address: String,
    task_id: u32,
}

pub_struct!(Serialize; Reward {
    task_id: u32,
    nft_contract: String,
    token_id: String,
    sig: (FieldElement, FieldElement),
});

pub_struct!(Serialize; RewardResponse {
    rewards: Vec<Reward>,
});

pub_struct!(Deserialize; VerifyQuery {
     addr: FieldElement,
});

pub_struct!(Deserialize; EmailQuery {
    addr: FieldElement,
    email: String,
});

pub_struct!(Deserialize; VerifyQuizQuery {
    addr: FieldElement,
    quiz_name: String,
    user_answers_list: Vec<Vec<String>>,
});

pub_struct!(Deserialize; AchievementQuery {
    addr: FieldElement,
});

pub_struct!(Deserialize; VerifyAchievementQuery {
    addr: FieldElement,
    id: u32,
});

pub_struct!(Debug, Serialize, Deserialize; AchievedDocument {
    addr: String,
    achievement_id: u32,
});

pub_struct!(Debug, Serialize, Deserialize; AchievementDocument {
    id: u32,
    category_id: u32,
    name: String,
    img_url: String,
    short_desc: String,
    todo_title: String,
    todo_desc: String,
    done_title: String,
    done_desc: String,
    verify_type: String,
    experience:i64,
});

pub_struct!(Debug, Serialize, Deserialize; AchievementCategoryDocument {
    id: u32,
    name: String,
    desc: String,
    img_url: String,
});

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAchievements {
    category_id: u32,
    category_name: String,
    category_desc: String,
    category_img_url: String,
    category_type: String,
    #[serde(default = "default_category_disabled")]
    pub category_disabled: bool,
    pub category_override_verified_type: Option<String>,
    achievements: Vec<UserAchievement>,
}

pub fn default_category_disabled() -> bool {
    false
}

pub_struct!(Debug, Serialize, Deserialize; UserAchievement {
    id: u32,
    name: String,
    short_desc: String,
    title: String,
    desc: String,
    completed: bool,
    verify_type: String,
    img_url: String,
});

pub_struct!(Debug, Serialize, Deserialize; UserExperience {
    address: String,
    experience:i64,
    timestamp:i64,
});

pub_struct!(Debug, Serialize, Deserialize; LeaderboardTable {
    experience:i64,
    timestamp:f64,
});

pub_struct!(Debug, Serialize, Deserialize; BoostTable {
    amount: i32,
    token: String,
    expiry: i64,
    quests: Vec<i32>,
    winner: Option<Vec<String>>,
    id: i32,
    img_url: String,
    name: String,
    hidden: bool,
    num_of_winners: i64,
    token_decimals: i64,
});

pub_struct!(Debug, Serialize, Deserialize; NftBalance {
    contract_address: String,
    token_id: String,
    owner_address: String,
    balance: String,
});

pub_struct!(Debug, Serialize, Deserialize; Nft {
    nft_id: String,
    contract_address: String,
    token_id: String,
    name: Option<String>,
    description: Option<String>,
    external_url: Option<String>,
    attributes: Option<Value>,
    image_url: Option<String>,
    image_small_url: Option<String>,
    image_medium_url: Option<String>,
    animation_url: Option<String>,
    minted_by_address: String,
    minted_at_transaction_hash: String,
    minted_at_timestamp: i64,
    balance: Option<NftBalance>,
});

pub_struct!(Debug, Serialize, Deserialize; StarkscanQuery {
    next_url: Option<String>,
    data: Vec<Nft>,
});

pub_struct!(Deserialize; BuildingQuery {
    ids: String,
});

pub_struct!(Debug, Deserialize, Serialize; BuildingDocument {
    id: u32,
    name: String,
    description: String,
    entity: String,
    level: u32,
    img_url: String,
});

pub_struct!(Deserialize, Debug; DeployedTime {
    addr: String,
    timestamp: u32,
});

pub_struct!(Deserialize; VerifyAchievementBatchedQuery {
    addr: FieldElement,
    category_id: u32,
});

pub_struct!(Deserialize, Serialize, Debug; UserAchievementsCategory {
    category_id: u32,
    achievements: Vec<UserAchievementCategory>,
});

pub_struct!(Deserialize, Serialize, Debug; UserAchievementCategory {
    id: u32,
    completed: bool,
    verify_type: String,
});

pub_struct!(Debug, Serialize, Deserialize; QuestCategoryDocument {
    name: String,
    title: String,
    desc: String,
    img_url: String,
});
