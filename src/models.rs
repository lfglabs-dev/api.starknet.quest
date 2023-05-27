use mongodb::Database;
use serde::{Deserialize, Serialize};
use starknet::providers::SequencerGatewayProvider;

use crate::config::Config;

pub struct AppState {
    pub conf: Config,
    pub provider: SequencerGatewayProvider,
    pub db: Database,
}

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
});

pub_struct!(Deserialize; CompletedTasks {
    address: String,
    task_id: u32,
});
