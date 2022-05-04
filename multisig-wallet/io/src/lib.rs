#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;
use codec::{Codec};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MWAction {
    AddOwner(ActorId),
    RemoveOwner(ActorId),
    ReplaceOwner { old_owner: ActorId, new_owner: ActorId },
    ChangeRequiredConfirmationsCount(usize),
    SubmitTransaction {
        destination: ActorId,
        data: Box<dyn Codec>,
        value: u128,
    },
    ConfirmTransaction(U256),
    RevokeConfirmation(U256),
    ExecuteTransaction(U256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MWEvent {
    Confirmation {
        sender: ActorId,
        transaction_id: U256,
    },
    Revocation {
        sender: ActorId,
        transaction_id: U256,
    },
    Submission {
        transaction_id: U256,
    },
    Execution {
        transaction_id: U256,
    },
    ExecutionFailure {
        transaction_id: U256,
    },
    Deposit {
        sender: ActorId,
    },
    OwnerAddition {
        owner: ActorId,
    },
    OwnerRemoval {
        owner: ActorId,
    },
    RequirementChange(U256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct MWInitConfig {
    pub owners: Box<[ActorId]>,
    pub required: usize,
}
