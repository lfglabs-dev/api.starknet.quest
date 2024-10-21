use mongodb::Database;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet::{
    core::types::FieldElement,
    providers::{jsonrpc::HttpTransport, JsonRpcClient},
};

use crate::endpoints::quests::uri::Attribute;
use crate::{config::Config, logger::Logger};
use tokio::sync::Mutex;

pub_struct!(;AppState {
    last_task_id: Mutex<i64>,
    conf: Config,
    provider: JsonRpcClient<HttpTransport>,
    db: Database,
    logger: Logger,
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
    hidden: Option<bool>,
    disabled: bool,
    expiry: Option<i64>,
    expiry_timestamp: Option<String>,
    mandatory_domain: Option<String>,
    expired: Option<bool>,
    experience: i64,
    start_time: i64,
});

pub_struct!(Debug, Serialize, Deserialize; QuestInsertDocument {
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
    disabled: bool,
    expiry: Option<i64>,
    mandatory_domain: Option<String>,
    experience: i32,
    start_time: i64,
});

pub_struct!(Debug, Serialize, Deserialize;  QuizInsertDocument {
    id: u32,
    name: String,
    desc: String,
   intro:String,
});

pub_struct!(Debug, Serialize, Deserialize; QuizQuestionDocument {
    id: i64,
    question: String,
    options:Vec<String>,
    correct_answers: Vec<i64>,
    kind: String,
    layout: String,
    quiz_id: i64,
});

pub_struct!(Serialize, Deserialize; NFTUri {
    id: i64,
    name: String,
    description:String,
    image: String,
    quest_id: i64,
    attributes: Option<Attribute>
});

pub_struct!(Deserialize; CompletedTasks {
    address: String,
    task_id: u32,
    timestamp: i64,
});

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTaskDocument {
    address: String,
    task_id: u32,
    timestamp: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Call {
    pub contract: String,
    pub call_data: Vec<String>,
    pub entry_point: String,
    pub regex: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractCall {
    pub contractaddress: String,
    pub calldata: Vec<String>,
    pub entrypoint: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QuestTaskDocument {
    pub(crate) id: i32,
    pub quest_id: i64,
    pub name: String,
    pub desc: String,
    pub cta: String,
    pub total_amount: Option<FieldElement>,
    pub verify_endpoint: String,
    pub href: String,
    pub verify_endpoint_type: String,
    pub calls: Option<Vec<Call>>,
    #[serde(default)]
    pub verify_redirect: Option<String>,
    #[serde(default)]
    pub quiz_name: Option<i64>,
    #[serde(default)]
    pub task_type: Option<String>,
    #[serde(default)]
    pub(crate) discord_guild_id: Option<String>,
    #[serde(default)]
    pub(crate) contracts: Option<Vec<FieldElement>>,
    pub api_url: Option<String>,
    pub regex: Option<String>,
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
    task_id: Option<u32>,
});

pub_struct!(Deserialize; VerifyNewQuery {
    addr: FieldElement,
    quest_id: i64,
    task_id: u32,
});

pub_struct!(Deserialize; EmailQuery {
    addr: FieldElement,
    email: String,
});

pub_struct!(Deserialize; VerifyQuizQuery {
    addr: FieldElement,
    quiz_name: i64,
    user_answers_list: Vec<Vec<usize>>,
});

pub_struct!(Deserialize; VerifyBalanceQuery {
    addr: FieldElement,
    task_id: u32,
});

pub_struct!(Deserialize; UniquePageVisit {
    viewer_ip: String,
    viewed_page_id: String,
    timestamp: i64,
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
    timestamp: i64,
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

pub_struct!(Debug, Serialize, Deserialize; JWTClaims {
    sub: String,
    exp: usize,
});

pub_struct!(Debug, Serialize, Deserialize; LoginDetails {
    user: String,
    code: String,
});

pub_struct!(Deserialize; CreateBoostQuery {
    quest_id: i32,
    amount: i32,
    token: String,
    num_of_winners: i64,
    token_decimals: i64,
    name: String,
    img_url: String,
    expiry: i64,
});

#[derive(Serialize, Deserialize, Debug)]
pub struct ZkLendReward {
    pub amount: Amount,
    pub claim_contract: FieldElement,
    pub claim_id: u64,
    pub claimed: bool,
    pub proof: Vec<String>,
    pub recipient: String,
    pub token: Token,
    #[serde(rename = "type")]
    pub response_type: String, // renaming to avoid keyword conflict
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Amount {
    pub decimals: u8,
    pub value: FieldElement,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
// Nostra Reward Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct NostraResponse {
    pub documents: Vec<NostraReward>, // Array of reward documents
}

// Nostra Reward Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct NostraPeriodsResponse {
    pub documents: Vec<NostraRewardPeriods>, // Array of reward documents
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NostraReward {
    #[serde(rename = "_id")]
    pub id_internal: String,
    pub id: String,
    pub account: String,
    pub proofs: Vec<String>,
    pub reward: FieldElement,
    pub reward_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NostraRewardPeriods {
    #[serde(rename = "_id")]
    pub id_internal: String,
    pub id: String,
    pub defi_spring_rewards: bool,
    pub defi_spring_rewards_distributor: Option<FieldElement>,
}

// Nimbora Reward Struct
#[derive(Serialize, Deserialize, Debug)]
pub struct NimboraRewards {
    pub amount: FieldElement,
    pub proof: Vec<String>,
}

// Ekubo Reward Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct EkuboRewards {
    pub contract_address: FieldElement,
    pub token: String,
    pub start_date: String,
    pub end_date: String,
    pub claim: Claim,
    pub proof: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claim {
    pub id: u64,
    pub amount: FieldElement,
    pub claimee: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RewardSource {
    ZkLend,
    Nostra,
    Nimbora,
    Ekubo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonReward {
    pub amount: FieldElement,
    pub displayed_amount: FieldElement,
    pub proof: Vec<String>,
    pub reward_id: Option<u64>,
    pub claim_contract: FieldElement,
    pub token_symbol: String,
    pub reward_source: RewardSource,
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefiReward {
    pub amount: FieldElement,
    pub token_symbol: String,
}
