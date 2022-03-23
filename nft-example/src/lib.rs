#![no_std]
#![feature(const_btree_new)]

use codec::Encode;
use gstd::{debug, msg, prelude::*, ActorId};
use gstd::any::Any;
use primitive_types::U256;

pub mod state;
pub use state::{State, StateReply};

pub use nft_example_io::{Action, Event, InitConfig};

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::NonFungibleToken;
type Ihello = fn() -> ();
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct NFT {
    pub token: NonFungibleToken,
    pub token_id: U256,
    pub owner: ActorId,
    pub functions_map: BTreeMap<String, Ihello>,
}

static mut CONTRACT: NFT = NFT {
    token: NonFungibleToken::new(),
    token_id: U256::zero(),
    owner: ZERO_ID,
    functions_map: BTreeMap::new(),
};

impl NFT {
    fn mint(&mut self) {
        self.token.owner_by_id.insert(self.token_id, msg::source());
        let balance = *self
            .token
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.token
            .balances
            .insert(msg::source(), balance.saturating_add(U256::one()));

        msg::reply(
            Event::Transfer {
                from: ZERO_ID,
                to: msg::source(),
                token_id: self.token_id,
            },
            0,
        );
        self.token_id = self.token_id.saturating_add(U256::one());
        self.functions_map.insert(
            "mint".to_string(),
            &self.mint()
        );
    }

    fn burn(&mut self, token_id: U256) {
        if !self.token.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.token.is_token_owner(token_id, &msg::source()) {
            panic!("NonFungibleToken: account is not owner");
        }
        self.token.token_approvals.remove(&token_id);
        self.token.owner_by_id.remove(&token_id);
        let balance = *self
            .token
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.token
            .balances
            .insert(msg::source(), balance.saturating_sub(U256::one()));
        msg::reply(
            Event::Transfer {
                from: msg::source(),
                to: ZERO_ID,
                token_id,
            },
            0,
        );
    }
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

pub trait Call {
    fn call(&self, action: &Action) -> ();
}

// macro_rules! call_action {
//     ($struct:ident, $action:ident) => {
//         impl Call for $struct {
//             fn call(&self, action: &Action) -> () {
//                 match $action {

//                 }
//                 ()
//             }
//         }
//     }
// }
//call_action!(NFT, Action);

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
  //  CONTRACT.call(&action);
  let s: String = "Hello, World".to_string();
    let any: Box<dyn Any> = Box::new(s);
   // dfdf!(action, contract);
    match action {
        Action::Mint => {
            CONTRACT.mint();
        }
        Action::Burn(amount) => {
            CONTRACT.burn(amount);
        }
        Action::Transfer { to, token_id } => {
            CONTRACT.token.transfer(&msg::source(), &to, token_id);
        }
        Action::Approve { to, token_id } => {
            CONTRACT.token.approve(&msg::source(), &to, token_id);
        }
        Action::ApproveForAll { to, approved } => {
            CONTRACT
                .token
                .approve_for_all(&msg::source(), &to, approved);
        }
        Action::OwnerOf(input) => {
            CONTRACT.token.owner_of(input);
        }
        Action::BalanceOf(input) => {
            CONTRACT.token.balance_of(&input);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NFT {:?}", config);
    CONTRACT
        .token
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.owner = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let encoded = match query {
        State::BalanceOfUser(input) => {
            StateReply::BalanceOfUser(*CONTRACT.token.balances.get(&input).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.token.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(*user).encode()
        }
        State::IsTokenOwner { account, token_id } => {
            let user = CONTRACT
                .token
                .owner_by_id
                .get(&token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &account).encode()
        }
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .token
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(*approved_address).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}
