use crate::{
    common::verify_has_root_or_braavos_domain::verify_has_root_or_braavos_domain,
    models::{AppState, VerifyQuery},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    verify_has_root_or_braavos_domain(state, &query.addr, 100).await
}
