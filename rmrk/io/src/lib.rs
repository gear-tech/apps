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

#[derive(Debug, Clone, Encode)]
pub struct Child {
    pub token_id: ActorId,
    pub status: ChildStatus,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Copy)]
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
    AcceptChild {
        parent_token_id: TokenId,
        child_token_id: TokenId,
    },
    TransferChildren {
        parent_token_id: TokenId,
        children_ids: Vec<TokenId>,
        children_token_ids: Vec<ActorId>,
        children_statuses: Vec<ChildStatus>,
        add: bool,
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
        root_owner: ActorId,
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
    NFTParent {
        parent: ActorId,
    },
    RootOwner {
        root_owner: ActorId,
    },
    TransferChildren {
        parent_token_id: TokenId,
        children_ids: Vec<TokenId>,
        children_token_ids: Vec<ActorId>,
        children_statuses: Vec<ChildStatus>,
        add: bool,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
}
