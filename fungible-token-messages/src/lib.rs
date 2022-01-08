#![no_std]

use codec::{Decode, Encode};
use gstd::prelude::*;
use primitive_types::H256;
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct MintInput {
    pub account: H256,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct BurnInput {
    pub account: H256,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveInput {
    pub spender: H256,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveReply {
    pub owner: H256,
    pub spender: H256,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferInput {
    pub to: H256,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferReply {
    pub from: H256,
    pub to: H256,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferFromInput {
    pub owner: H256,
    pub to: H256,
    pub amount: u128,
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
    TotalIssuance,
    BalanceOf(H256),
    AddAdmin(H256),
    RemoveAdmin(H256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer(TransferReply),
    Approval(ApproveReply),
    TotalIssuance(u128),
    Balance(u128),
    AdminAdded(H256),
    AdminRemoved(H256),
}
