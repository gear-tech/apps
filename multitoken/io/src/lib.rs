#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::multitoken::io::*;
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MyMTKAction {
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
    BalanceOf {
        account: ActorId,
        id: TokenId,
    },
    BalanceOfBatch {
        accounts: Vec<ActorId>,
        ids: Vec<TokenId>,
    },
    MintBatch {
        amounts: Vec<u128>,
        ids: Vec<TokenId>,
        tokens_metadata: Vec<Option<TokenMetadata>>,
    },
    TransferFrom {
        from: ActorId,
        to: ActorId,
        id: TokenId,
        amount: u128,
    },
    BatchTransferFrom {
        from: ActorId,
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<u128>,
    },
    BurnBatch {
        ids: Vec<TokenId>,
        amounts: Vec<u128>,
    },
    Approve {
        account: ActorId,
    },
    RevokeApproval {
        account: ActorId,
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
