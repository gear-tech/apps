#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::erc1155::io::*;
use gstd::{prelude::*};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ERC1155Action {
    Base(Action),
    Mint {
        amount: u128,
        token_metadata: Option<TokenMetadata>,
    },
    Burn {
        id: TokenId,
        amount: u128,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitERC1155 {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
