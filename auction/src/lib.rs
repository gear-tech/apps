#![no_std]
#![feature(const_btree_new)]

use codec::Encode;
use gstd::{debug, msg, prelude::*, exec::block_timestamp, ActorId};
use primitive_types::U256;

pub mod state;
pub use state::{State, StateReply};

pub use auction_io::{Action, Event, InitConfig};

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::NonFungibleToken;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const DURATION: u64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Debug)]
pub struct NFT {
    pub token: NonFungibleToken,
    pub token_id: U256,
    pub owner: ActorId,
}

#[derive(Debug)]
pub struct Auction {
    pub nft: NFT,
    pub starting_price: U256,
    pub discount_rate: U256,
    pub already_bought: bool,
    pub start_at: u64,
    pub expires_at: u64,
}

static mut CONTRACT: Auction = Auction {
    nft: NFT {
        token: NonFungibleToken::new(),
        token_id: U256::zero(),
        owner: ZERO_ID,
    },
    starting_price: U256::zero(),
    discount_rate: U256::zero(),
    already_bought: false,
    start_at: 0,
    expires_at: 0,
};

impl Auction {
    fn buy(&mut self) {
        if self.already_bought {
            panic!("already bought");
        }

        if block_timestamp() >= self.expires_at {
            panic!("auction expired");
        }

        let price = self.token_price();

        if U256::from(msg::value()) < price {
            panic!("value < price");
        }

        self.already_bought = true;

        let refund = msg::value() - price.as_u128();

        msg::reply(
            Event::Transfer {
                from: self.nft.owner,
                to: msg::source(),
                token_id: self.nft.token_id,
            },
            refund,
        );
    }

    fn token_price(&self) -> U256 {
        let time_elapsed = block_timestamp() - self.start_at;
        let discount = self.discount_rate * time_elapsed;

        self.starting_price - discount
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
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("Auction {:?}", config);

    if config.starting_price < config.discount_rate * DURATION {
        panic!("starting price < min");
    }

    CONTRACT
        .nft
        .token
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.start_at = block_timestamp();
    CONTRACT.expires_at = block_timestamp() + DURATION;
    CONTRACT.nft.owner = msg::source();
    CONTRACT.nft.token_id = config.token_id;
    CONTRACT.discount_rate = config.discount_rate;
    CONTRACT.starting_price = config.starting_price;
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Buy => {
            CONTRACT.buy();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let encoded = match query {
        State::TokenPrice() => {
            let price = CONTRACT.token_price();
            StateReply::TokenPrice(price)
        }
    }.encode();
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}
