use serde::Deserialize;
use starknet::core::types::FieldElement;

#[derive(Deserialize)]
pub struct ScoreResponse {
    pub owner: String,
    pub score: f64,
}
