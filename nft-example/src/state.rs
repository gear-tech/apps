use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    BalanceOfUser(ActorId),
    TokenOwner(U256),
    IsTokenOwner(TokenAndUser),
    GetApproved(U256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum StateReply {
    BalanceOfUser(U256),
    TokenOwner(ActorId),
    IsTokenOwner(bool),
    GetApproved(ActorId),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TokenAndUser {
    pub token_id: U256,
    pub user: ActorId,
}
