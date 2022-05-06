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
    MintToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    MintToRootOwner {
        to: ActorId,
        token_id: TokenId,
    },
    AddChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_token_address: ActorId,
    },
    NFTParent {
        token_id: TokenId,
    },
    CheckRMRKImplementation,
    RootOwner {token_id: TokenId},
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum RMRKEvent {
    MintToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    MintToRootOwner {
        to: ActorId,
        token_id: TokenId,
    },
    PendingChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    ChildAdded,
    NFTParent { parent: ActorId },
    CheckRMRKImplementation,
    RootOwner { root_owner: ActorId }
}
