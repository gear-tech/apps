#![no_std]
#![feature(const_btree_new)]
#![feature(specialization)]
use codec::Encode;
use gstd::{debug, msg, prelude::*, ActorId};
use primitive_types::U256;

pub mod state;
pub use state::{State, StateReply};
pub use nft_example_io::{Action, Event, InitConfig};

pub mod base;
use base::*;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
pub use nft_derive::NFTStorage;


pub trait NFTStorage {
    fn get(&self) -> &NFT;
    fn get_mut(&mut self) -> &mut NFT;
}

#[derive(Debug)]
pub struct NFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub owner_by_id: BTreeMap<U256, ActorId>,
  //  pub token_metadata_by_id: BTreeMap<U256, TokenMetadata>,
    pub token_approvals: BTreeMap<U256, ActorId>,
    pub balances: BTreeMap<ActorId, U256>,
    pub operator_approval: BTreeMap<ActorId, ActorId>,
}

// static mut CONTRACT: NFT = NFT {
//     token: NonFungibleToken::new(),
//     token_id: U256::zero(),
//     owner: ZERO_ID,
// };

impl <T: NFTStorage> NonFungibleTokenBase for T{
    default fn transfer(&mut self, to: &ActorId, token_id: U256) {
        let from_balance = *self.get_mut().balances.get(&msg::source()).unwrap_or(&U256::zero());
      //  let to_balance = *self.balances.get(to).unwrap_or(&U256::zero());

    }

    default fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256) {
        let from_balance = *self.get_mut().balances.get(&msg::source()).unwrap_or(&U256::zero());
    }

    default fn balance_of(&self, to: &ActorId) {

    }

    default fn owner_of(&self, token_id: U256) {

    }


    // fn mint(&mut self) {
    //     self.token.owner_by_id.insert(self.token_id, msg::source());
    //     let balance = *self
    //         .token
    //         .balances
    //         .get(&msg::source())
    //         .unwrap_or(&U256::zero());
    //     self.token
    //         .balances
    //         .insert(msg::source(), balance.saturating_add(U256::one()));

    //     msg::reply(
    //         Event::Transfer {
    //             from: ZERO_ID,
    //             to: msg::source(),
    //             token_id: self.token_id,
    //         },
    //         0,
    //     );
    //     self.token_id = self.token_id.saturating_add(U256::one());
    // }

    // fn burn(&mut self, token_id: U256) {
    //     if !self.token.exists(token_id) {
    //         panic!("NonFungibleToken: Token does not exist");
    //     }
    //     if !self.token.is_token_owner(token_id, &msg::source()) {
    //         panic!("NonFungibleToken: account is not owner");
    //     }
    //     self.token.token_approvals.remove(&token_id);
    //     self.token.owner_by_id.remove(&token_id);
    //     let balance = *self
    //         .token
    //         .balances
    //         .get(&msg::source())
    //         .unwrap_or(&U256::zero());
    //     self.token
    //         .balances
    //         .insert(msg::source(), balance.saturating_sub(U256::one()));
    //     msg::reply(
    //         Event::Transfer {
    //             from: msg::source(),
    //             to: ZERO_ID,
    //             token_id,
    //         },
    //         0,
    //     );
    // }
}

gstd::metadata! {
    title: "NftExample",
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
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    // match action {
    //     Action::Mint => {
    //         CONTRACT.mint();
    //     }
    //     Action::Burn(amount) => {
    //         CONTRACT.burn(amount);
    //     }
    //     Action::Transfer { to, token_id } => {
    //         CONTRACT.token.transfer(&msg::source(), &to, token_id);
    //     }
    //     Action::Approve { to, token_id } => {
    //         CONTRACT.token.approve(&msg::source(), &to, token_id);
    //     }
    //     Action::ApproveForAll { to, approved } => {
    //         CONTRACT
    //             .token
    //             .approve_for_all(&msg::source(), &to, approved);
    //     }
    //     Action::OwnerOf(input) => {
    //         CONTRACT.token.owner_of(input);
    //     }
    //     Action::BalanceOf(input) => {
    //         CONTRACT.token.balance_of(&input);
    //     }
    // }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NFT {:?}", config);
//     CONTRACT
//         .token
//         .init(config.name, config.symbol, config.base_uri);
//     CONTRACT.owner = msg::source();
 }

