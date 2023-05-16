use mongodb::Database;

use crate::config::Config;

pub struct AppState {
    pub conf: Config,
    pub db: Database,
}
