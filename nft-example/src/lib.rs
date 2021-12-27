#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

pub mod payloads;
<<<<<<< HEAD
<<<<<<< HEAD
pub use payloads::{ApproveForAllInput, ApproveInput, InitConfig, TransferInput};

pub mod state;
pub use state::{State, StateReply};

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::{Approve, ApproveForAll, NonFungibleToken, Transfer};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());
=======
pub use payloads::{InitConfig, TransferInput, ApproveInput, ApproveForAllInput};
=======
pub use payloads::{ApproveForAllInput, ApproveInput, InitConfig, TransferInput};
>>>>>>> created nft-example using nft-library and nft-example test

pub mod state;
pub use state::{State, StateReply};

use non_fungible_token::base::NonFungibleTokenBase;
<<<<<<< HEAD
>>>>>>> draft nft-library and api for nft-example
=======
use non_fungible_token::{Approve, ApproveForAll, NonFungibleToken, Transfer};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());
>>>>>>> created nft-example using nft-library and nft-example test

#[derive(Debug, Decode, TypeInfo)]
pub enum Action {
    Mint,
    Burn(U256),
    Transfer(TransferInput),
    Approve(ApproveInput),
    ApproveForAll(ApproveForAllInput),
<<<<<<< HEAD
<<<<<<< HEAD
    OwnerOf(U256),
    BalanceOf(H256),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    // Base(NftEvent),
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug)]
pub struct NFT {
    pub tokens: NonFungibleToken,
    pub token_id: U256,
=======
    UpdateRoyalty(Royalties),
=======
    OwnerOf(U256),
    BalanceOf(H256),
>>>>>>> created nft-example using nft-library and nft-example test
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    // Base(NftEvent),
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug)]
pub struct NFT {
    pub tokens: NonFungibleToken,
<<<<<<< HEAD
    pub royalties: Royalties,
>>>>>>> draft nft-library and api for nft-example
=======
    pub token_id: U256,
>>>>>>> created nft-example using nft-library and nft-example test
    pub owner: ActorId,
}

static mut CONTRACT: NFT = NFT {
<<<<<<< HEAD
<<<<<<< HEAD
    tokens: NonFungibleToken {
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
        self.tokens.owner_by_id.insert(self.token_id, msg::source());
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
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
        if !self.tokens.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.tokens.is_token_owner(token_id, &msg::source()) {
            panic!("NonFungibleToken: account is not owner");
        }
        self.tokens.token_approvals.remove(&token_id);
        self.tokens.owner_by_id.remove(&token_id);
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
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
=======
    tokens: NonFungibleToken{
=======
    tokens: NonFungibleToken {
>>>>>>> created nft-example using nft-library and nft-example test
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
        self.tokens.owner_by_id.insert(self.token_id, msg::source());
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
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
        if !self.tokens.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.tokens.is_token_owner(token_id, &msg::source()) {
            panic!("NonFungibleToken: account is not owner");
        }
        self.tokens.token_approvals.remove(&token_id);
        self.tokens.owner_by_id.remove(&token_id);
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
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
<<<<<<< HEAD

>>>>>>> draft nft-library and api for nft-example
=======
>>>>>>> created nft-example using nft-library and nft-example test
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
<<<<<<< HEAD
<<<<<<< HEAD
            CONTRACT.mint();
        }
        Action::Burn(input) => {
            CONTRACT.burn(input);
        }
        Action::Transfer(input) => {
            CONTRACT.tokens.transfer(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
=======
           CONTRACT.mint();
=======
            CONTRACT.mint();
>>>>>>> created nft-example using nft-library and nft-example test
        }
        Action::Burn(input) => {
            CONTRACT.burn(input);
        }
        Action::Transfer(input) => {
<<<<<<< HEAD
            CONTRACT.tokens.transfer_from(
                &ActorId::new(input.from.to_fixed_bytes()),
                &ActorId::new(input.to.to_fixed_bytes()),  
                input.token_id
>>>>>>> draft nft-library and api for nft-example
=======
            CONTRACT.tokens.transfer(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
>>>>>>> created nft-example using nft-library and nft-example test
            );
        }
        Action::Approve(input) => {
            CONTRACT.tokens.approve(
<<<<<<< HEAD
<<<<<<< HEAD
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
            );
        }
        Action::ApproveForAll(input) => {
            CONTRACT.tokens.approve_for_all(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.approve,
            );
        }
        Action::OwnerOf(input) => {
            let owner = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            msg::reply(
                Event::OwnerOf(H256::from_slice(owner.as_ref())),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        Action::BalanceOf(input) => {
            let balance = *CONTRACT
                .tokens
                .balances
                .get(&ActorId::new(input.to_fixed_bytes()))
                .unwrap_or(&U256::zero());
            msg::reply(
                Event::BalanceOf(balance),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
=======
                &ActorId::new(input.to.to_fixed_bytes()),  
=======
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
>>>>>>> created nft-example using nft-library and nft-example test
                input.token_id,
            );
        }
        Action::ApproveForAll(input) => {
            CONTRACT.tokens.approve_for_all(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.approve,
            );
        }
<<<<<<< HEAD

        Action::UpdateRoyalty(input) => {
            CONTRACT.set_royalties(input)
>>>>>>> draft nft-library and api for nft-example
=======
        Action::OwnerOf(input) => {
            let owner = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            msg::reply(
                Event::OwnerOf(H256::from_slice(owner.as_ref())),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        Action::BalanceOf(input) => {
            let balance = *CONTRACT
                .tokens
                .balances
                .get(&ActorId::new(input.to_fixed_bytes()))
                .unwrap_or(&U256::zero());
            msg::reply(
                Event::BalanceOf(balance),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
>>>>>>> created nft-example using nft-library and nft-example test
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NFT {:?}", config);
<<<<<<< HEAD
<<<<<<< HEAD
    CONTRACT
        .tokens
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.owner = msg::source();
=======
    CONTRACT.tokens.set_name(config.name);
    CONTRACT.tokens.set_symbol(config.symbol);
    CONTRACT.tokens.set_base_uri(config.base_uri);
    CONTRACT.owner = msg::source();
    debug!(
        "NON_FUNGIBLE_TOKEN created"
    );
>>>>>>> draft nft-library and api for nft-example
=======
    CONTRACT
        .tokens
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.owner = msg::source();
>>>>>>> created nft-example using nft-library and nft-example test
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
<<<<<<< HEAD
<<<<<<< HEAD
    let encoded = match query {
        State::BalanceOfUser(input) => {
            let user = &ActorId::new(input.to_fixed_bytes());
            StateReply::BalanceOfUser(*CONTRACT.tokens.balances.get(user).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(H256::from_slice(user.as_ref())).encode()
        }
        State::IsTokenOwner(input) => {
            let user = CONTRACT
                .tokens
                .owner_by_id
                .get(&input.token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &ActorId::new(input.user.to_fixed_bytes())).encode()
        }
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .tokens
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(H256::from_slice(approved_address.as_ref())).encode()
=======
    let zero = ActorId::new(H256::zero().to_fixed_bytes());
=======
>>>>>>> created nft-example using nft-library and nft-example test
    let encoded = match query {
        State::BalanceOfUser(input) => {
            let user = &ActorId::new(input.to_fixed_bytes());
            StateReply::BalanceOfUser(*CONTRACT.tokens.balances.get(user).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(H256::from_slice(user.as_ref())).encode()
        }
        State::IsTokenOwner(input) => {
            let user = CONTRACT
                .tokens
                .owner_by_id
                .get(&input.token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &ActorId::new(input.user.to_fixed_bytes())).encode()
        }
<<<<<<< HEAD
       State::GetApproved(input) => {
            let approved_address = CONTRACT.tokens.token_approvals.get(&input).unwrap_or(&zero);
            StateReply::GetApproved(
                H256::from_slice(approved_address.as_ref())
            ).encode()      
>>>>>>> draft nft-library and api for nft-example
=======
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .tokens
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(H256::from_slice(approved_address.as_ref())).encode()
>>>>>>> created nft-example using nft-library and nft-example test
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
<<<<<<< HEAD
<<<<<<< HEAD
}
=======
}
>>>>>>> draft nft-library and api for nft-example
=======
}
>>>>>>> created nft-example using nft-library and nft-example test
