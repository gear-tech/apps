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

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum ChildStatus {
    Pending,
    Accepted,
    Unknown,
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
    Burn {
        token_id: TokenId,
    },
    BurnChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    TransferToNft {
        to: ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    },
    Approve {
        to: ActorId,
        token_id: TokenId,
    },
    AddChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    AddChildAccepted {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    AcceptChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    RejectChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    RemoveChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },

    NFTParent {
        token_id: TokenId,
    },
    RootOwner {
        token_id: TokenId,
    },
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
    Approval {
        owner: ActorId,
        approved_account: ActorId,
        token_id: TokenId,
    },
    PendingChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    AcceptedChild {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    ChildAdded {
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_status: ChildStatus,
    },
    ChildBurnt {
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_status: ChildStatus,
    },
    PendingChildRemoved {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    ChildRemoved {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    ChildRejected {
        child_token_address: ActorId,
        child_token_id: TokenId,
        parent_token_id: TokenId,
    },
    NFTParent {
        parent: ActorId,
    },
    RootOwner {
        root_owner: ActorId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
}
