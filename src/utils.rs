use crate::models::{AppState, CompletedTasks};
use async_trait::async_trait;
use axum::{
    http::StatusCode,
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
use std::fmt::Write;
use std::result::Result;

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

pub fn to_hex(felt: FieldElement) -> String {
    let bytes = felt.to_bytes_be();
    let mut result = String::with_capacity(bytes.len() * 2 + 2);
    result.push_str("0x");
    for byte in bytes {
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}
