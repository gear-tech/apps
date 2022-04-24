#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::erc1155::io::*;
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertAction {
    Create {
        contract_id: ActorId,
        creator: ActorId,
        concert_id: u128,
        no_tickets: u128,
        // date: u128,
    },
    Hold {
        concert_id: u128,
    },
    BuyTicket {
        concert_id: u128,
        amount: u128,
        metadata: Vec<Option<TokenMetadata>>,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertEvent {
    Creation {
        creator: ActorId,
        concert_id: u128,
        no_tickets: u128,
    },
    Hold {
        concert_id: u128,
    },
    Purchase {
        concert_id: u128,
        amount: u128,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConcert {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
