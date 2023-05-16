use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CompletedTasks {
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

#[derive(Serialize)]
pub struct QueryError {
    pub error: String,
    pub res: bool,
}
