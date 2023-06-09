use crate::models::{AppState, CompletedTasks};
use async_trait::async_trait;
use axum::body::Body;
use axum::{
    http::{Response as HttpResponse, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use mongodb::{bson::doc, options::UpdateOptions, results::UpdateResult, Collection};
use starknet::signers::Signer;
use starknet::{
    core::{
        crypto::{pedersen_hash, Signature},
        types::FieldElement,
    },
    signers::LocalWallet,
};
use std::result::Result;
use std::str::FromStr;

#[macro_export]
macro_rules! pub_struct {
    ($($derive:path),*; $name:ident {$($field:ident: $t:ty),* $(,)?}) => {
        #[derive($($derive),*)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub async fn get_nft(
    quest_id: u32,
    task_id: u32,
    addr: &FieldElement,
    nft_level: u32,
    signer: &LocalWallet,
) -> Result<(u64, Signature), Box<dyn std::error::Error + Send + Sync>> {
    let token_id = nft_level as u64 + 100 * (rand::random::<u64>() % (2u64.pow(32)));
    let hashed = pedersen_hash(
        &pedersen_hash(
            &pedersen_hash(
                &pedersen_hash(&FieldElement::from(token_id), &FieldElement::ZERO),
                &FieldElement::from(quest_id),
            ),
            &FieldElement::from(task_id),
        ),
        addr,
    );
    let sig = signer.sign_hash(&hashed).await?;
    Ok((token_id, sig))
}

pub fn get_error(error: String) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
}

pub fn get_error_redirect(redirect_uri: String, error: String) -> Response {
    let err_msg_encoded =
        percent_encoding::utf8_percent_encode(&error, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    let redirect_url = format!("{}&error_msg={}", redirect_uri, err_msg_encoded);
    let uri = match Uri::from_str(&redirect_url) {
        Ok(uri) => uri,
        Err(_) => return get_error("Failed to create URI from redirect URL".to_string()),
    };

    let response = match HttpResponse::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::from("Redirecting..."))
    {
        Ok(response) => response,
        Err(_) => return get_error("Failed to create HTTP response".to_string()),
    };

    response.into_response()
}

pub fn success_redirect(redirect_uri: String) -> Response {
    let uri = match Uri::from_str(&redirect_uri) {
        Ok(uri) => uri,
        Err(_) => return get_error("Failed to create URI from redirect URL".to_string()),
    };

    let response = match HttpResponse::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", uri.to_string())
        .body(Body::from("Redirecting..."))
    {
        Ok(response) => response,
        Err(_) => return get_error("Failed to create HTTP response".to_string()),
    };

    response.into_response()
}

#[async_trait]
pub trait CompletedTasksTrait {
    async fn upsert_completed_task(
        &self,
        addr: FieldElement,
        task_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error>;
}

#[async_trait]
impl CompletedTasksTrait for AppState {
    async fn upsert_completed_task(
        &self,
        addr: FieldElement,
        task_id: u32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let completed_tasks_collection: Collection<CompletedTasks> =
            self.db.collection("completed_tasks");
        let filter = doc! { "address": addr.to_string(), "task_id": task_id };
        let update = doc! { "$setOnInsert": { "address": addr.to_string(), "task_id": task_id } };
        let options = UpdateOptions::builder().upsert(true).build();

        let result = completed_tasks_collection
            .update_one(filter, update, options)
            .await;
        result
    }
}
