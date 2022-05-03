#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::non_fungible_token::token::*;
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint { token_metadata: TokenMetadata },
    Burn { token_id: TokenId },
    Transfer { to: ActorId, token_id: TokenId },
    Approve { to: ActorId, token_id: TokenId },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
