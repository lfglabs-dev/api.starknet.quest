use crate::models::Nft;
use regex::Regex;

pub fn is_braavos_whitelisted(nft: &Nft, unique_nfts: &mut Vec<String>) {
    let whitelist_patterns = vec![
        r"Starknet Onboarding Journey( NFT)?",
        r"Starknet Identity Journey",
        r"Starknet Exchange Journey",
        r"Starknet Mobile Journey",
        r"(Starknet Journey Coin NFT|starknet-journey-coin)",
    ];
    if let Some(name) = nft.name.as_ref() {
        for pattern in &whitelist_patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(name) && !unique_nfts.contains(name) {
                unique_nfts.push(name.clone());
                return;
            }
        }
    }
}

pub fn is_argent_whitelisted(nft: &Nft, unique_nfts: &mut Vec<String>) {
    if let Some(name) = nft.name.as_ref() {
        if !unique_nfts.contains(name) {
            unique_nfts.push(name.to_string());
        }
    }
}

pub fn is_carbonable_whitelisted(nft: &Nft, unique_nfts: &mut Vec<String>) {
    if let Some(name) = nft.name.as_ref() {
        if !unique_nfts.contains(name) {
            unique_nfts.push(name.to_string());
        }
    }
}
