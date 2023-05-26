use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedTaskDocument {
    address: String,
    task_id: u32,
}

#[derive(Deserialize)]
pub struct ScoreResponse {
    pub owner: String,
    pub score: f64,
}

#[derive(Deserialize)]
pub struct StarkfighterQuery {
    pub addr: String,
}
