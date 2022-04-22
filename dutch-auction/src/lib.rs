#![no_std]
#![feature(const_btree_new)]

use codec::Encode;
use gstd::{msg, prelude::*, exec::block_timestamp, ActorId};
use primitive_types::U256;
use nft_example_io;

pub mod state;
pub use state::{State, StateReply, AuctionInfo,};

pub use auction_io::{Action, Event, InitConfig, CreateConfig};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const DURATION: u64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Debug)]
pub struct NFT {
    pub token_id: U256,
    pub owner: ActorId,
    pub contract_id: ActorId,
}

#[derive(Debug)]
pub struct Auction {
    pub nft: NFT,
    pub starting_price: U256,
    pub discount_rate: U256,
    pub is_active: bool,
    pub start_at: u64,
    pub expires_at: u64,
}

static mut CONTRACT: Auction = Auction {
    nft: NFT {
        token_id: U256::zero(),
        owner: ZERO_ID,
        contract_id: ZERO_ID,
    },
    starting_price: U256::zero(),
    discount_rate: U256::zero(),
    is_active: false,
    start_at: 0,
    expires_at: 0,
};

impl Auction {
    fn buy(&mut self) {
        if !self.is_active {
            panic!("already bought or auction expired");
        }

        if block_timestamp() >= self.expires_at {
            panic!("auction expired");
        }

        let price = self.token_price().as_u128();

        if msg::value() < price {
            panic!("value < price");
        }

        self.is_active = false;
        let refund = msg::value() - price;

        msg::send_and_wait_for_reply(
            self.nft.contract_id,
            nft_example_io::Action::Transfer { to: msg::source(), token_id: self.nft.token_id },
            0
        );
        msg::send_and_wait_for_reply(msg::source(), "", refund);
        msg::send_and_wait_for_reply(self.nft.owner, "", price);
    }

    fn token_price(&self) -> U256 {
        let time_elapsed = block_timestamp() - self.start_at;
        let discount = self.discount_rate * time_elapsed;

        self.starting_price - discount
    }

    fn renew_contract(&mut self, config: CreateConfig) {
        if self.is_active {
            panic!("already in use")
        }

        if config.starting_price < config.discount_rate * DURATION {
            panic!("starting price < min");
        }

        self.is_active = true;
        self.start_at = block_timestamp();
        self.expires_at = block_timestamp() + DURATION;
        self.nft.token_id = config.token_id;
        self.nft.contract_id = config.nft_contract_actor_id;
        self.nft.owner = config.token_owner;
        self.discount_rate = config.discount_rate;
        self.starting_price = config.starting_price;

        msg::reply(
            Event::AuctionStarted {
                token_owner: self.nft.owner,
                price: self.starting_price,
                token_id: self.nft.token_id,
            },
            0,
        );
    }

    fn stop_if_time_is_over(&mut self) {
        if block_timestamp() >= self.expires_at {
            self.is_active = false
        }
    }

    fn info(&self) -> AuctionInfo {
        AuctionInfo {
            nft_contract_actor_id: self.nft.contract_id,
            token_id: self.nft.token_id,
            token_owner: self.nft.owner,
            starting_price: self.starting_price,
        }
    }
}

gstd::metadata! {
    title: "Auction",
        init:
            input: InitConfig,
        handle:
            input: Action,
            output: Event,
        state:
            input: State,
            output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() { }

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");

    CONTRACT.stop_if_time_is_over();

    match action {
        Action::Buy => CONTRACT.buy(),
        Action::Create(config) => CONTRACT.renew_contract(config),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");

    CONTRACT.stop_if_time_is_over();

    let encoded = match query {
        State::TokenPrice() => StateReply::TokenPrice(CONTRACT.token_price()),
        State::IsActive() => StateReply::IsActive(CONTRACT.is_active),
        State::Info() => StateReply::Info(CONTRACT.info()),
    }.encode();
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}
