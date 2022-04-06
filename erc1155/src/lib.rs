#![no_std]
#![feature(const_btree_new)]

#[cfg(test)]
use codec::Encode;
use gstd::{debug, exec, msg, prelude::*, ActorId};

pub mod base;
use base::{ERC1155TokenBase, ExtendERC1155TokenBase};

pub mod common;
use common::*;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct ERC1155Token {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub balances: BTreeMap<TokenId, BTreeMap<ActorId, u128>>,
    pub operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
    pub token_metadata: BTreeMap<TokenId, TokenMetadata>,
}

static mut ERC1155_TOKEN: ERC1155Token = ERC1155Token {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    balances: BTreeMap::new(),
    operator_approvals: BTreeMap::new(),
    token_metadata: BTreeMap::new(),
};

impl ERC1155Token {
    fn get_balance(&self, account: &ActorId, id: &TokenId) -> u128 {
        *self
            .balances
            .get(id)
            .and_then(|m| m.get(account))
            .unwrap_or(&0)
    }

    fn set_balance(&mut self, account: &ActorId, id: &TokenId, amount: u128) {
        debug!(
            "before mint: {:?}, id: {:?}",
            self.get_balance(account, id),
            id
        );

        let mut _balance = self
            .balances
            .entry(*id)
            .or_default()
            .insert(*account, amount);

        debug!(
            "after mint: {:?}, id: {:?}",
            self.get_balance(account, id),
            id
        );
    }
}

impl ERC1155TokenBase for ERC1155Token {
    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[TokenId]) -> Vec<BalanceOfBatchReply> {
        if accounts.len() != ids.len() {
            panic!("ERC1155: accounts and ids length mismatch")
        }

        ids.iter()
            .zip(accounts)
            .map(|(id, account)| BalanceOfBatchReply {
                account: *account,
                id: *id,
                amount: self.get_balance(account, id),
            })
            .collect()
    }

    fn set_approval_for_all(&mut self, operator: &ActorId, approved: bool) {
        let owner = msg::source();

        if owner == *operator {
            panic!("ERC1155: setting approval status for self")
        }

        self.operator_approvals
            .entry(owner)
            .or_default()
            .insert(*operator, approved);

        msg::reply(
            Event::ApprovalForAll {
                owner: msg::source(),
                operator: *operator,
                approved,
            },
            0,
        );
    }

    fn is_approved_for_all(&self, owner: &ActorId, operator: &ActorId) -> bool {
        self.operator_approvals.contains_key(owner)
            && *self.operator_approvals[owner]
                .get(operator)
                .unwrap_or(&false)
    }

    fn transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &TokenId, amount: u128) {
        if from == to {
            panic!("ERC1155: sender and recipient addresses are the same")
        }

        if !(from == &msg::source() || self.is_approved_for_all(from, &msg::source())) {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        let from_balance = self.get_balance(from, id);

        if from_balance < amount {
            panic!("ERC1155: insufficient balance for transfer")
        }
        self.set_balance(from, id, from_balance.saturating_sub(amount));
        let to_balance = self.get_balance(to, id);
        self.set_balance(to, id, to_balance.saturating_add(amount));

        msg::reply(
            Event::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: *from,
                to: *to,
                id: *id,
                amount,
            }),
            0,
        );
    }

    fn batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
    ) {
        if from == to {
            panic!("ERC1155: sender and recipient addresses are the same")
        }

        if !(from == &msg::source() || self.is_approved_for_all(from, &msg::source())) {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            if !self.can_transfer(from, id, *amount) {
                panic!("ERC1155: all batch element should be transerfable");
            }
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.transfer_from(from, to, id, amounts[i]));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: *from,
                to: *to,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        );
    }

    fn can_transfer(&self, from: &ActorId, id: &u128, amount: u128) -> bool {
        from == &msg::source()
            || from == &exec::origin()
            || self.get_balance(&msg::source(), id) >= amount
    }
}

impl ExtendERC1155TokenBase for ERC1155Token {
    fn is_owner_of(&self, id: &TokenId) -> bool {
        let owner = msg::source();
        self.get_balance(&owner, id) != 0
    }

    fn is_owner_of_batch(&self, ids: &[TokenId]) -> bool {
        for ele in ids {
            if !self.is_owner_of(ele) {
                return false;
            }
        }
        true
    }

    fn mint(&mut self, account: &ActorId, id: &TokenId, amount: u128, meta: Option<TokenMetadata>) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }
        if let Some(metadata) = meta {
            // Off you go...
            if amount > 1 {
                panic!("ERC1155: Mint metadata to a fungible token")
            }
            self.token_metadata.insert(*id, metadata);
        }
        let prev_balance = self.get_balance(account, id);
        self.set_balance(account, id, prev_balance.saturating_add(amount));
        msg::reply(
            Event::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: ZERO_ID,
                to: *account,
                id: *id,
                amount,
            }),
            0,
        );
    }

    fn mint_batch(
        &mut self,
        account: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
        meta: Vec<Option<TokenMetadata>>,
    ) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }
        meta.into_iter()
            .enumerate()
            .for_each(|(i, meta)| self.mint(account, &ids[i], amounts[i], meta));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: ZERO_ID,
                to: *account,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        );
    }

    fn burn(&mut self, id: &TokenId, amount: u128) {
        let owner = &msg::source();
        if self.can_burn(owner, id, amount) {
            self.set_balance(
                &msg::source(),
                id,
                self.get_balance(owner, id).saturating_sub(amount),
            );
        }

        msg::reply(
            Event::TransferSingle(TransferSingleReply {
                operator: msg::source(),
                from: msg::source(),
                to: ZERO_ID,
                id: *id,
                amount,
            }),
            0,
        );
    }

    fn burn_batch(&mut self, ids: &[TokenId], amounts: &[u128]) {
        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (id, amount) in ids.iter().zip(amounts) {
            if !self.can_burn(&msg::source(), id, *amount) {
                panic!("ERC1155: all batch element should be burnable");
            }
        }

        ids.iter()
            .enumerate()
            .for_each(|(i, id)| self.burn(id, amounts[i]));

        msg::reply(
            Event::TransferBatch {
                operator: msg::source(),
                from: msg::source(),
                to: ZERO_ID,
                ids: ids.to_vec(),
                values: amounts.to_vec(),
            },
            0,
        );
    }

    fn uri(&self, id: TokenId) -> String {
        self.base_uri.clone().replace("{id}", &format!("{}", id))
    }

    fn can_burn(&mut self, owner: &ActorId, id: &TokenId, amount: u128) -> bool {
        self.is_owner_of(id) && self.get_balance(owner, id) >= amount
    }

    fn get_metadata(&self, id: TokenId) -> TokenMetadata {
        self.token_metadata
            .get(&id)
            .unwrap_or(&TokenMetadata {
                ..Default::default()
            })
            .clone()
    }
}

gstd::metadata! {
    title: "ERC1155",
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

    ERC1155_TOKEN.name = config.name;
    ERC1155_TOKEN.symbol = config.symbol;
    ERC1155_TOKEN.base_uri = config.base_uri;
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");

    let encoded = match query {
        State::Name => StateReply::Name(ERC1155_TOKEN.name.clone()).encode(),
        State::Symbol => StateReply::Name(ERC1155_TOKEN.symbol.clone()).encode(),
        State::Uri => StateReply::Uri(ERC1155_TOKEN.base_uri.clone()).encode(),
        State::BalanceOf(account, id) => {
            StateReply::Balance(ERC1155_TOKEN.get_balance(&account, &id)).encode()
        }
        State::URI(id) => StateReply::URI(ERC1155_TOKEN.uri(id)).encode(),
        State::MetadataOf(id) => StateReply::MetadataOf(ERC1155_TOKEN.get_metadata(id)).encode(),
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));
    core::mem::forget(encoded);
    result
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Mint(account, id, amount, meta) => {
            ERC1155_TOKEN.mint(&account, &id, amount, meta);
        }

        Action::BalanceOf(account, id) => {
            let balance = ERC1155_TOKEN.get_balance(&account, &id);
            msg::reply(Event::Balance(balance), 0);
        }

        Action::BalanceOfBatch(accounts, ids) => {
            let res = ERC1155_TOKEN.balance_of_batch(&accounts, &ids);
            msg::reply(Event::BalanceOfBatch(res), 0);
        }

        Action::MintBatch(account, ids, amounts, metas) => {
            ERC1155_TOKEN.mint_batch(&account, &ids, &amounts, metas);
        }

        Action::SafeTransferFrom(from, to, id, amount) => {
            ERC1155_TOKEN.transfer_from(&from, &to, &id, amount);
        }

        Action::SafeBatchTransferFrom(from, to, ids, amounts) => {
            ERC1155_TOKEN.batch_transfer_from(&from, &to, &ids, &amounts);
        }

        Action::SetApprovalForAll(operator, approved) => {
            ERC1155_TOKEN.set_approval_for_all(&operator, approved);
        }

        Action::IsApprovedForAll(owner, operator) => {
            let approved = ERC1155_TOKEN.is_approved_for_all(&owner, &operator);
            msg::reply(
                Event::ApprovalForAll {
                    owner,
                    operator,
                    approved,
                },
                0,
            );
        }

        Action::Burn(id, amount) => {
            ERC1155_TOKEN.burn(&id, amount);
        }

        Action::BurnBatch(ids, amounts) => {
            ERC1155_TOKEN.burn_batch(&ids, &amounts);
        }

        Action::OwnerOf(id) => {
            let res = ERC1155_TOKEN.is_owner_of(&id);
            msg::reply(res, 0);
        }

        Action::OwnerOfBatch(ids) => {
            let res = ERC1155_TOKEN.is_owner_of_batch(&ids);
            msg::reply(res, 0);
        }
    }
}
