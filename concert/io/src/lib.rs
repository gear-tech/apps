#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

// This is afrom contract, remove later to imports :)
pub type TokenId = u128;

#[derive(Debug, Decode, Encode, TypeInfo, Default, Clone)]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum ConcertAction {
    Create {
        creator: ActorId,
        concert_id: u128,
        no_tickets: u128,
        // date: u128,
    },
    Hold {
        concert_id: u128,
    },
    BuyTickets {
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
    pub owner_id: ActorId,
    pub mtk_contract: ActorId,
}
