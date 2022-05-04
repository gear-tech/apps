#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;
pub type TokenId = U256;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitRMRK {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum RMRKAction {
    AddChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_token_address: ActorId,
    },
    NFTParent {
        token_id: TokenId,
    },
    CheckRMRKImplementation,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum RMRKEvent {
    ChildAdded,
    NFTParent { parent: ActorId },
    CheckRMRKImplementation,
}
