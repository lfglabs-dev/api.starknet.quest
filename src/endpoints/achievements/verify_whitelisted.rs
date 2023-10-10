use crate::models::Nft;
use regex::Regex;

lazy_static::lazy_static! {
    static ref BRAAVOS_WHITELIST : Vec<Regex> = vec![
        Regex::new(r"Starknet Onboarding Journey( NFT)?").unwrap(),
        Regex::new(r"Starknet Identity Journey").unwrap(),
        Regex::new(r"Starknet Exchange Journey").unwrap(),
        Regex::new(r"Starknet Mobile Journey").unwrap(),
        Regex::new(r"(Starknet Journey Coin NFT|starknet-journey-coin)").unwrap()
    ];
}

pub fn is_braavos_whitelisted(nft: &Nft, unique_nfts: &mut Vec<String>) {
    if let Some(name) = nft.name.as_ref() {
        for pattern in &*BRAAVOS_WHITELIST {
            if pattern.is_match(name) && !unique_nfts.contains(name) {
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
