#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    Buy,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: U256,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {
    pub starting_price: U256,
    pub discount_rate: U256,
    pub token_id: U256,

    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
