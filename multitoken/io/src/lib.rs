#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::multitoken::io::*;
use gstd::prelude::*;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MyMTKAction {
    Base(MTKAction),
    Mint {
        amount: u128,
        token_metadata: Option<TokenMetadata>,
    },
    Burn {
        id: TokenId,
        amount: u128,
    },
    Supply {
        id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MyMTKEvent {
    Supply { amount: u128 },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitMTK {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
