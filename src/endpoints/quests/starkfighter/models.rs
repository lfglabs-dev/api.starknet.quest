use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScoreResponse {
    pub owner: String,
    pub score: f64,
}
