use crate::{Item, ContractAndTokenId};
use codec::{Decode, Encode};
use gstd::{prelude::*};
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    AllItems,
    ItemInfo(ContractAndTokenId),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    AllItems (Vec<Item>),
    ItemInfo (Item),
}