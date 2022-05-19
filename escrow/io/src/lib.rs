#![no_std]

use gstd::{prelude::*, ActorId};
use primitive_types::U256;

/// Escrow account ID.
pub type AccountId = U256;

#[derive(Decode, Encode, TypeInfo)]
pub struct InitEscrow {
    /// Address of a fungible token program.
    pub ft_program_id: ActorId,
}

#[derive(Decode, Encode, TypeInfo)]
pub enum EscrowAction {
    Create {
        buyer: ActorId,
        seller: ActorId,
        amount: u128,
    },
    Deposit(AccountId),
    Confirm(AccountId),
    Refund(AccountId),
    Cancel(AccountId),
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
    Created(AccountId),
}

#[derive(Decode, Encode, TypeInfo)]
pub enum EscrowState {
    GetInfo(AccountId),
}

#[derive(Decode, Encode, TypeInfo)]
pub enum EscrowStateReply {
    Info(Account),
}

#[derive(Decode, Encode, TypeInfo, Clone, Copy)]
pub struct Account {
    pub buyer: ActorId,
    pub seller: ActorId,
    pub state: AccountState,
    pub amount: u128,
}

#[derive(Decode, Encode, TypeInfo, PartialEq, Clone, Copy)]
pub enum AccountState {
    AwaitingDeposit,
    AwaitingConfirmation,
    Closed,
}
