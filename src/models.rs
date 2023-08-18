use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};
use starknet::{core::types::FieldElement, providers::SequencerGatewayProvider};

use crate::config::Config;

pub_struct!(;AppState {
    conf: Config,
    provider: SequencerGatewayProvider,
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
    issuer: String,
    category: String,
    rewards_endpoint: String,
    logo: String,
    rewards_img: String,
    rewards_title: String,
    rewards_nfts: Vec<NFTItem>,
    img_card: String,
    title_card: String,
    hidden: bool,
    disabled: bool,
    expiry: Option<bson::DateTime>,
    expiry_timestamp: Option<String>,
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
    verify_endpoint: String,
});

pub_struct!(Debug, Serialize, Deserialize; AchievementCategoryDocument {
    id: u32,
    name: String,
    desc: String,
});

pub_struct!(Debug, Serialize, Deserialize; UserAchievements {
    category_name: String,
    category_desc: String,
    achievements: Vec<UserAchievement>,
});

pub_struct!(Debug, Serialize, Deserialize; UserAchievement {
    name: String,
    short_desc: String,
    title: String,
    desc: String,
    completed: bool,
    verify_type: String,
});
