#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
pub mod base;
use base::NonFungibleTokenBase;
pub mod token;
use token::TokenMetadata;

use primitive_types::U256;
use scale_info::TypeInfo;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct NonFungibleToken {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub owner_by_id: BTreeMap<U256, ActorId>,
    pub token_metadata_by_id: BTreeMap<U256, TokenMetadata>,
    pub token_approvals: BTreeMap<U256, ActorId>,
    pub balances: BTreeMap<ActorId, U256>,
    pub operator_approval: BTreeMap<ActorId, ActorId>,
}

impl NonFungibleTokenBase for NonFungibleToken {
    fn init(&mut self, name: String, symbol: String, base_uri: String) {
        self.name = name;
        self.symbol = symbol;
        self.base_uri = base_uri;
    }

    fn transfer(&mut self, from: &ActorId, to: &ActorId, token_id: U256) {
        if !self.exists(token_id) {
            panic!("NonFungibleToken: token does not exist");
        }
        if from == to {
            panic!("NonFungibleToken: Transfer to current owner");
        }
        if to == &ZERO_ID {
            panic!("NonFungibleToken: Transfer to zero address.");
        }

        match self.authorized_actor(token_id, from) {
            AuthAccount::None => {
                panic!("NonFungibleToken: is not an authorized source");
            }
            AuthAccount::ApprovedActor => {
                self.token_approvals.remove(&token_id);
            }
            _ => {}
        }

        let from_balance = *self.balances.get(from).unwrap_or(&U256::zero());
        let to_balance = *self.balances.get(to).unwrap_or(&U256::zero());

        self.balances
            .insert(*from, from_balance.saturating_sub(U256::one()));
        self.balances
            .insert(*to, to_balance.saturating_add(U256::one()));

        self.owner_by_id.insert(token_id, *to);

        msg::reply(
            Event::Transfer {
                from: *from,
                to: *to,
                token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256) {
        if spender == &ZERO_ID {
            panic!("NonFungibleToken: Approval to zero address.");
        }
        if spender == owner {
            panic!("NonFungibleToken: Approval to current owner");
        }
        if !self.is_token_owner(token_id, owner) {
            panic!("NonFungibleToken: is not owner");
        }

        self.token_approvals.insert(token_id, *spender);

        msg::reply(
            Event::Approval {
                owner: *owner,
                spender: *spender,
                token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn approve_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool) {
        if operator == &ZERO_ID {
            panic!("NonFungibleToken: Approval for a zero address");
        }
        match approved {
            true => self.operator_approval.insert(*owner, *operator),
            false => self.operator_approval.remove(owner),
        };

        msg::reply(
            Event::ApprovalForAll {
                owner: *owner,
                operator: *operator,
                approved,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn balance_of(&self, account: &ActorId) {
        let balance = *self.balances.get(account).unwrap_or(&U256::zero());
        msg::reply(
            Event::BalanceOf(balance),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn owner_of(&self, token_id: U256) {
        let owner = self.owner_by_id.get(&token_id).unwrap_or(&ZERO_ID);
        msg::reply(
            Event::OwnerOf(*owner),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

impl NonFungibleToken {
    pub const fn new() -> NonFungibleToken {
        NonFungibleToken {
            name: String::new(),
            symbol: String::new(),
            base_uri: String::new(),
            owner_by_id: BTreeMap::new(),
            token_metadata_by_id: BTreeMap::new(),
            token_approvals: BTreeMap::new(),
            balances: BTreeMap::new(),
            operator_approval: BTreeMap::new(),
        }
    }

    pub fn is_token_owner(&self, token_id: U256, account: &ActorId) -> bool {
        account == self.owner_by_id.get(&token_id).unwrap_or(&ZERO_ID)
    }

    pub fn authorized_actor(&self, token_id: U256, account: &ActorId) -> AuthAccount {
        let owner = self.owner_by_id.get(&token_id).unwrap_or(&ZERO_ID);
        if owner == account {
            return AuthAccount::Owner;
        }
        if self.token_approvals.get(&token_id).unwrap_or(&ZERO_ID) == account {
            return AuthAccount::ApprovedActor;
        }
        if self.operator_approval.contains_key(owner) {
            return AuthAccount::Operator;
        }
        AuthAccount::None
    }

    pub fn exists(&self, token_id: U256) -> bool {
        self.owner_by_id.contains_key(&token_id)
    }
}

#[derive(Debug, Encode, TypeInfo, Decode)]
pub enum Event {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: U256,
    },
    Approval {
        owner: ActorId,
        spender: ActorId,
        token_id: U256,
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
    OwnerOf(ActorId),
    BalanceOf(U256),
}

#[derive(Debug, Encode, TypeInfo)]
pub enum AuthAccount {
    Owner,
    ApprovedActor,
    Operator,
    None,
}
