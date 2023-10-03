use crate::models::Nft;

pub fn is_braavos_whitelisted(nft: &Nft) -> bool {
    let whitelist = vec![
        "Starknet Onboarding Journey NFT",
        "Starknet Identity Journey",
        "Starknet Exchange Journey",
        "Starknet Mobile Journey",
        "Starknet Journey Coin NFT",
    ];
    if let Some(name) = nft.name.as_ref() {
        return whitelist.contains(&name.as_str());
    }
    false
}

pub fn is_argent_whitelisted(_nft: &Nft) -> bool {
    true
}

pub fn is_carbonable_whitelisted(_nft: &Nft) -> bool {
    true
}
