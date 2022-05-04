use codec::{Decode, Encode};
use gstd::{ActorId, prelude::Vec};
use scale_info::TypeInfo;
use primitive_types::U256;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    ConfirmationsCount(U256),
    TransactionsCount {
        pending: bool,
        executed: bool,
    },
    Owners,
    Confirmations(U256),
    TransactionIds {
        from_index: usize,
        to_index: usize,
        pending: bool,
        executed: bool,
    },
    IsConfirmed(U256),
}

#[derive(Debug, Encode, TypeInfo)]
pub enum StateReply {
    ConfirmationCount(usize),
    TransactionsCount(usize),
    Owners(Vec<ActorId>),
    Confirmations(Vec<ActorId>),
    TransactionIds(Vec<U256>),
    IsConfirmed(bool),
}
