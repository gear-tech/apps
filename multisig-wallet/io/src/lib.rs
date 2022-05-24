#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MWAction {
    AddOwner(ActorId),
    RemoveOwner(ActorId),
    ReplaceOwner {
        old_owner: ActorId,
        new_owner: ActorId,
    },
    ChangeRequiredConfirmationsCount(u64),
    SubmitTransaction {
        destination: ActorId,
        data: Vec<u8>,
        value: u128,
        description: Option<String>,
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
    OwnerAddition {
        owner: ActorId,
    },
    OwnerRemoval {
        owner: ActorId,
    },
    OwnerReplace {
        old_owner: ActorId,
        new_owner: ActorId,
    },
    RequirementChange(U256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct MWInitConfig {
    pub owners: Vec<ActorId>,
    pub required: u64,
}
