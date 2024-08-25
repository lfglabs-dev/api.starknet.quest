use crate::{
    common::verify_has_root_or_braavos_domain::verify_has_root_or_braavos_domain,
    models::{AppState, VerifyQuery},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_auto_routes::route;
use std::sync::Arc;

#[route(get, "/quests/braavos/pyramid/verify_has_domain")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    verify_has_root_or_braavos_domain(state, &query.addr, 104).await
}
