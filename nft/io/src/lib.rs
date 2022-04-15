#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::non_fungible_token::{io::*, token::*};
use gstd::prelude::*;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MyNFTAction {
    Base(NFTAction),
    Mint { token_metadata: TokenMetadata },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
