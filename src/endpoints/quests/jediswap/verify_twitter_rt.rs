use std::sync::Arc;

use crate::{
    models::{AppState, VerifyQuery},
    utils::{get_error_redirect, success_redirect, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let quest_id = 2;
    let task_id = 11;
    let error_redirect_uri = format!(
        "{}/quest/{}?task_id={}&res=false",
        state.conf.variables.app_link, quest_id, task_id
    );
    match state.upsert_completed_task(query.addr, task_id).await {
        Ok(_) => success_redirect(format!(
            "{}/quest/{}?task_id={}&res=true",
            state.conf.variables.app_link, quest_id, task_id
        )),
        Err(e) => get_error_redirect(error_redirect_uri, format!("{}", e)),
    }
}
