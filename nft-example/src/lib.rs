#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

pub mod payloads;
pub use payloads::{ApproveForAllInput, ApproveInput, InitConfig, TransferInput};

pub mod state;
pub use state::{State, StateReply};

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::{Approve, ApproveForAll, NonFungibleToken, Transfer};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug, Decode, TypeInfo)]
pub enum Action {
    Mint,
    Burn(U256),
    Transfer(TransferInput),
    Approve(ApproveInput),
    ApproveForAll(ApproveForAllInput),
    OwnerOf(U256),
    BalanceOf(H256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug)]
pub struct NFT {
    pub token: NonFungibleToken,
    pub token_id: U256,
    pub owner: ActorId,
}

static mut CONTRACT: NFT = NFT {
    token: NonFungibleToken {
        name: String::new(),
        symbol: String::new(),
        base_uri: String::new(),
        owner_by_id: BTreeMap::new(),
        token_metadata_by_id: BTreeMap::new(),
        token_approvals: BTreeMap::new(),
        balances: BTreeMap::new(),
        operator_approval: BTreeMap::new(),
    },
    token_id: U256::zero(),
    owner: ActorId::new(H256::zero().to_fixed_bytes()),
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

        let transfer_token = Transfer {
            from: H256::zero(),
            to: H256::from_slice(msg::source().as_ref()),
            token_id: self.token_id,
        };

        self.token_id = self.token_id.saturating_add(U256::one());

        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
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

        let transfer_token = Transfer {
            from: H256::from_slice(msg::source().as_ref()),
            to: H256::zero(),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
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

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Mint => {
            CONTRACT.mint();
        }
        Action::Burn(input) => {
            CONTRACT.burn(input);
        }
        Action::Transfer(input) => {
            CONTRACT.token.transfer(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
            );
        }
        Action::Approve(input) => {
            CONTRACT.token.approve(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
            );
        }
        Action::ApproveForAll(input) => {
            CONTRACT.token.approve_for_all(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.approve,
            );
        }
        Action::OwnerOf(input) => {
            CONTRACT.token.owner_of(input);
        }
        Action::BalanceOf(input) => {
            CONTRACT
                .token
                .balance_of(&ActorId::new(input.to_fixed_bytes()));
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
            let user = &ActorId::new(input.to_fixed_bytes());
            StateReply::BalanceOfUser(*CONTRACT.token.balances.get(user).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.token.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(H256::from_slice(user.as_ref())).encode()
        }
        State::IsTokenOwner(input) => {
            let user = CONTRACT
                .token
                .owner_by_id
                .get(&input.token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &ActorId::new(input.user.to_fixed_bytes())).encode()
        }
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .token
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(H256::from_slice(approved_address.as_ref())).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}
