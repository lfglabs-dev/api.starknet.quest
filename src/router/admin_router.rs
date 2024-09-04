use std::sync::Arc;

use super::admin_routes;
use crate::models::AppState;
use axum::Router;

// It just nests the sub routers into itself.
pub fn router(state: Arc<AppState>) -> axum::Router {
    Router::new()
        .nest("/admin", admin_routes::routes())
        .with_state(state)
}