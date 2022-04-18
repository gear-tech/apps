use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
pub type TokenId = U256;

#[derive(Debug, Default, Decode, Encode, TypeInfo)]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: ActorId,
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Vec<ActorId>,
}

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
pub struct TokenMetadata {
    // ex. "CryptoKitty #100"
    pub title: Option<String>,
    // free-form description
    pub description: Option<String>,
    // URL to associated media, preferably to decentralized, content-addressed storage
    pub media: Option<String>,
    // URL to an off-chain JSON file with more info.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
}
