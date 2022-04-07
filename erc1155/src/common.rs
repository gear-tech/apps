use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;
pub type TokenId = u128;

#[derive(Debug, Decode, Encode, TypeInfo, Default, Clone)]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum State {
    Name,
    Symbol,
    Uri,
    BalanceOf(ActorId, u128),
    URI(TokenId),
    MetadataOf(TokenId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    Name(String),
    Symbol(String),
    Uri(String),
    Balance(TokenId),
    URI(String),
    MetadataOf(TokenMetadata),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Mint(ActorId, TokenId, u128, Option<TokenMetadata>),
    BalanceOf(ActorId, TokenId),
    BalanceOfBatch(Vec<ActorId>, Vec<TokenId>),
    MintBatch(ActorId, Vec<u128>, Vec<TokenId>, Vec<Option<TokenMetadata>>),
    TransferFrom(ActorId, ActorId, TokenId, u128),
    BatchTransferFrom(ActorId, ActorId, Vec<TokenId>, Vec<u128>),
    SetApprovalForAll(ActorId, bool),
    IsApprovedForAll(ActorId, ActorId),
    Burn(TokenId, u128),
    BurnBatch(Vec<TokenId>, Vec<u128>),
    OwnerOf(TokenId),
    OwnerOfBatch(Vec<TokenId>),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferSingleReply {
    pub operator: ActorId,
    pub from: ActorId,
    pub to: ActorId,
    pub id: TokenId,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct BalanceOfBatchReply {
    pub account: ActorId,
    pub id: TokenId,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    TransferSingle(TransferSingleReply),
    Balance(u128),
    BalanceOfBatch(Vec<BalanceOfBatchReply>),
    MintOfBatch(Vec<BalanceOfBatchReply>),
    TransferBatch {
        operator: ActorId,
        from: ActorId,
        to: ActorId,
        ids: Vec<TokenId>,
        values: Vec<u128>,
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
}
