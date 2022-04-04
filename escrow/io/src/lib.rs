#![no_std]

use gstd::{prelude::*, ActorId};

#[derive(Decode, Encode)]
pub struct InitConfig {
    pub buyer: ActorId,
    pub seller: ActorId,
    pub ft_program_id: ActorId,
    pub amount: u128,
}

#[derive(Decode, Encode)]
pub enum Action {
    Deposit,
    ConfirmDelivery,
}

#[derive(Decode, Encode)]
pub enum Event {
    Deposit { buyer: ActorId, amount: u128 },
    ConfirmDelivery { seller: ActorId, amount: u128 },
}
