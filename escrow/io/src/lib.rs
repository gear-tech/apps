#![no_std]

use gstd::{prelude::*, ActorId};
use primitive_types::U256;

#[derive(Decode, Encode, TypeInfo)]
pub struct InitEscrow {
    pub ft_program_id: ActorId,
}

#[derive(Decode, Encode, TypeInfo)]
pub enum EscrowAction {
    Create {
        buyer: ActorId,
        seller: ActorId,
        amount: u128,
    },
    Deposit(U256),
    Confirm(U256),
    Refund(U256),
    Cancel(U256),
}

#[derive(Decode, Encode, TypeInfo)]
pub enum EscrowEvent {
    Cancelled {
        buyer: ActorId,
        seller: ActorId,
        amount: u128,
    },
    Refunded {
        buyer: ActorId,
        amount: u128,
    },
    Confirmed {
        seller: ActorId,
        amount: u128,
    },
    Deposited {
        buyer: ActorId,
        amount: u128,
    },
    Created(U256),
}
