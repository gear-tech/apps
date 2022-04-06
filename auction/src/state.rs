use codec::{Decode, Encode};
use gstd::{prelude::*};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    TokenPrice(),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum StateReply {
    TokenPrice(U256),
}