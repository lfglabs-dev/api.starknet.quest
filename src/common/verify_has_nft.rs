use crate::{
    config::Config,
    models::{Nft, StarkscanQuery},
    utils::to_hex,
};
use starknet::core::types::FieldElement;

pub async fn execute_has_nft(
    config: &Config,
    addr: FieldElement,
    contract: FieldElement,
    limit: u32,
    is_whitelisted: fn(&Nft) -> bool,
) -> Result<bool, String> {
    let url = format!(
        "https://api.starkscan.co/api/v0/nfts?contract_address={}&owner_address={}",
        to_hex(contract),
        to_hex(addr)
    );
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("accept", "application/json")
        .header("x-api-key", config.starkscan.api_key.clone())
        .send()
        .await
    {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    match serde_json::from_str::<StarkscanQuery>(&text) {
                        Ok(res) => {
                            // Remove duplicates & check is whitelisted
                            let nft_data = res.data;
                            let mut unique_nfts: Vec<String> = Vec::new();
                            for nft in nft_data {
                                if nft.name.is_some() && is_whitelisted(&nft) {
                                    let name = nft.name.unwrap();
                                    if !unique_nfts.contains(&name) {
                                        unique_nfts.push(name);
                                    }
                                }
                            }
                            Ok(unique_nfts.len() >= limit as usize)
                        }
                        Err(e) => Err(format!(
                            "Failed to deserialize result from Starkscan API: {} for response: {}",
                            e, text
                        )),
                    }
                }
                Err(e) => Err(format!(
                    "Failed to get JSON response while fetching user NFT data: {}",
                    e
                )),
            }
        }
        Err(e) => Err(format!("Failed to fetch user NFTs from API: {}", e)),
    }
}
