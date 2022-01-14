#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
pub mod base;
use base::NonFungibleTokenBase;
pub mod token;
use token::TokenMetadata;

use primitive_types::{H256, U256};
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
    pub operator_approval: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
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

        let transfer_token = Transfer {
            from: H256::from_slice(from.as_ref()),
            to: H256::from_slice(to.as_ref()),
            token_id,
        };

        msg::reply(
            NftEvent::Transfer(transfer_token),
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

        let approve_token = Approve {
            owner: H256::from_slice(owner.as_ref()),
            spender: H256::from_slice(spender.as_ref()),
            token_id,
        };
        msg::reply(
            NftEvent::Approval(approve_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn approve_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool) {
        if operator == &ZERO_ID {
            panic!("NonFungibleToken: Approval for a zero address");
        }

        self.operator_approval
            .entry(*owner)
            .or_default()
            .insert(*operator, approved);

        let approve_operator = ApproveForAll {
            owner: H256::from_slice(owner.as_ref()),
            operator: H256::from_slice(operator.as_ref()),
            approved,
        };

        msg::reply(
            NftEvent::ApprovalForAll(approve_operator),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

impl NonFungibleToken {
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
        if *self
            .operator_approval
            .get(owner)
            .unwrap_or(&BTreeMap::<ActorId, bool>::default())
            .get(account)
            .unwrap_or(&false)
        {
            return AuthAccount::Operator;
        }
        AuthAccount::None
    }

    pub fn exists(&self, token_id: U256) -> bool {
        self.owner_by_id.contains_key(&token_id)
    }
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct Approve {
    owner: H256,
    spender: H256,
    token_id: U256,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveForAll {
    owner: H256,
    operator: H256,
    approved: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct Transfer {
    pub from: H256,
    pub to: H256,
    pub token_id: U256,
}

#[derive(Debug, Encode, TypeInfo, Decode)]
pub enum NftEvent {
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug, Encode, TypeInfo, Decode)]
pub enum AuthAccount {
    Owner,
    ApprovedActor,
    Operator,
    None,
}
