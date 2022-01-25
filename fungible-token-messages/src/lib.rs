#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct MintInput {
    pub account: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct BurnInput {
    pub account: ActorId,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveInput {
    pub spender: ActorId,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveReply {
    pub owner: ActorId,
    pub spender: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferInput {
    pub to: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferReply {
    pub from: ActorId,
    pub to: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferFromInput {
    pub owner: ActorId,
    pub to: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferFromReply {
    pub owner: ActorId,
    pub sender: ActorId,
    pub recipient: ActorId,
    pub amount: u128,
    pub new_limit: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct AllowanceInput {
    pub owner: ActorId,
    pub spender: ActorId,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct AllowanceReply {
    pub owner: ActorId,
    pub spender: ActorId,
    pub limit: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Mint(MintInput),
    Burn(BurnInput),
    Transfer(TransferInput),
    TransferFrom(TransferFromInput),
    Approve(ApproveInput),
    IncreaseAllowance(ApproveInput),
    DecreaseAllowance(ApproveInput),
    AddAdmin(ActorId),
    RemoveAdmin(ActorId),
    TotalSupply,
    BalanceOf(ActorId),
    Allowance(AllowanceInput),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer(TransferReply),
    Approval(ApproveReply),
    AdminAdded(ActorId),
    AdminRemoved(ActorId),
    TransferFrom(TransferFromReply),
    TotalSupply(u128),
    Balance(u128),
    Allowance(AllowanceReply),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum State {
    Name,
    Symbol,
    Decimals,
    TotalSupply,
    BalanceOf(ActorId),
    Allowance(AllowanceInput),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    Name(String),
    Symbol(String),
    Decimals(u8),
    TotalSupply(u128),
    Balance(u128),
    Allowance(AllowanceReply),
}
