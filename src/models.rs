use mongodb::Database;
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
    finished: bool,
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
