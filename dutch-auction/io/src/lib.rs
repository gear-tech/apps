#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    Buy,
    Create(CreateConfig),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    AuctionStarted {
        token_owner: ActorId,
        price: U256,
        token_id: U256,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct CreateConfig {
    pub nft_contract_actor_id: ActorId,
    pub token_owner: ActorId,
    pub token_id: U256,
    pub starting_price: U256,
    pub discount_rate: U256,
}
