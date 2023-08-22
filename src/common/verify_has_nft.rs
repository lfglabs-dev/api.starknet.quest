use crate::{config::Config, models::StarkscanQuery, utils::to_hex};
use starknet::core::types::FieldElement;

pub async fn execute_has_nft(
    config: &Config,
    addr: FieldElement,
    contract: FieldElement,
    limit: u32,
) -> bool {
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
                            // Remove duplicates
                            let nft_data = res.data;
                            let mut unique_nfts: Vec<String> = Vec::new();
                            for nft in nft_data {
                                if nft.name.is_some() {
                                    let name = nft.name.unwrap();
                                    if !unique_nfts.contains(&name) {
                                        unique_nfts.push(name);
                                    }
                                }
                            }
                            unique_nfts.len() >= limit as usize
                        }
                        Err(e) => {
                            println!("Failed to deserialize result from Starkscan API: {}", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to get JSON response while fetching user NFT data: {}",
                        e
                    );
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch user NFTs from API: {}", e);
            false
        }
    }
}
