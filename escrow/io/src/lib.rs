#![no_std]

use gstd::{prelude::*, ActorId};

#[derive(Decode, Encode)]
pub struct InitConfig {
    pub ft_program_id: ActorId,
}

#[derive(Decode, Encode)]
pub enum Action {
    Create {
        buyer: ActorId,
        seller: ActorId,
        amount: u128,
    },
    Deposit {
        contract_id: u128,
    },
    Confirm {
        contract_id: u128,
    },
    Refund {
        contract_id: u128,
    },
    Cancel {
        contract_id: u128,
    },
}

#[derive(Decode, Encode, TypeInfo)]
pub enum Event {
    Cancelled {
        buyer: ActorId,
        seller: ActorId,
        amount: u128,
    },
    Refunded {
        amount: u128,
        buyer: ActorId,
    },
    Confirmed {
        amount: u128,
        seller: ActorId,
    },
    Deposited {
        buyer: ActorId,
        amount: u128,
    },
    Created { contract_id: u128 }
}
