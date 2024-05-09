use crate::{
    common::verify_has_root_domain::execute_has_root_domain,
    models::{AppState, VerifyQuery},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_auto_routes::route;
use std::sync::Arc;

#[route(
    get,
    "/quests/orbiter/verify_has_root_domain",
    crate::endpoints::quests::orbiter::verify_has_root_domain
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    execute_has_root_domain(state, &query.addr, 33).await
}
