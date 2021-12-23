use scale_info::TypeInfo;
use primitive_types::{H256, U256};
use codec::{Decode, Encode};
use gstd::{String};

#[derive(Debug, Decode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferInput {
    pub from: H256,
    pub to: H256,
    pub token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ApproveInput {
    pub to: H256,
    pub token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ApproveForAllInput {
    pub to: H256,
    pub approve: bool,
}