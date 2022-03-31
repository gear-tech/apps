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
    pub balances: BTreeMap<u128, BTreeMap<ActorId, u128>>,
    pub operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

static mut ERC1155_TOKEN: ERC1155Token = ERC1155Token {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    balances: BTreeMap::new(),
    operator_approvals: BTreeMap::new(),
};

impl ERC1155Token {
    fn get_balance(&self, account: &ActorId, id: &u128) -> u128 {
        *self
            .balances
            .get(id)
            .and_then(|m| m.get(account))
            .unwrap_or(&0)
    }

    fn set_balance(&mut self, account: &ActorId, id: &u128, amount: u128) {
        debug!(
            "before mint: {:?}, id: {:?}",
            self.balance_of(account, id),
            id
        );

        let mut _balance = self
            .balances
            .entry(*id)
            .or_default()
            .insert(*account, amount);

        debug!(
            "after mint: {:?}, id: {:?}",
            self.balance_of(account, id),
            id
        );
    }
}

impl ERC1155TokenBase for ERC1155Token {
    fn balance_of(&self, account: &ActorId, id: &u128) -> u128 {
        *self
            .balances
            .get(id)
            .and_then(|m| m.get(account))
            .unwrap_or(&0)
    }

    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[u128]) -> Vec<BalanceOfBatchReply> {
        if accounts.len() != ids.len() {
            panic!("ERC1155: accounts and ids length mismatch")
        }

        ids.iter()
            .enumerate()
            .map(|(_s, x)| BalanceOfBatchReply {
                account: accounts[_s],
                id: *x,
                amount: *self
                    .balances
                    .get(x)
                    .and_then(|m| m.get(&accounts[_s]))
                    .unwrap_or(&0),
            })
            .collect::<Vec<BalanceOfBatchReply>>()
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
    }

    fn is_approved_for_all(&self, owner: &ActorId, operator: &ActorId) -> bool {
        self.operator_approvals.contains_key(owner)
            && *self.operator_approvals[owner]
                .get(operator)
                .unwrap_or(&false)
    }

    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &u128, amount: u128) {
        if from == to {
            panic!("ERC1155: sender and recipient addresses are the same")
        }

        if !(from == &msg::source() || self.is_approved_for_all(from, &msg::source())) {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        let from_balance = self.balance_of(from, id);

        if from_balance < amount {
            panic!("ERC1155: insufficient balance for transfer")
        }
        self.set_balance(from, id, from_balance.saturating_sub(amount));
        let to_balance = self.balance_of(to, id);
        self.set_balance(to, id, to_balance.saturating_add(amount));
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[u128],
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

        ids.iter().enumerate().for_each(|(_s, x)| {
            if !self.can_transfer(from, x, amounts[_s]) {
                panic!("ERC1155: all batch element should be transerfable");
            }
        });

        ids.iter()
            .enumerate()
            .for_each(|(_s, x)| self.safe_transfer_from(from, to, x, amounts[_s]));
    }

    fn can_transfer(&mut self, from: &ActorId, id: &u128, amount: u128) -> bool {
        if from == &msg::source()
            || from == &exec::origin()
            || self.get_balance(&msg::source(), id) >= amount
        {
            return true;
        }
        false
    }
}

impl ExtendERC1155TokenBase for ERC1155Token {
    fn owner_of(&self, id: &u128) -> bool {
        let owner = msg::source();

        self.balance_of(&owner, id) != 0
    }

    fn owner_of_batch(&self, ids: &[u128]) -> bool {
        for (_, ele) in ids.iter().enumerate() {
            let res = self.owner_of(ele);
            if !res {
                return false;
            }
        }

        true
    }

    fn mint(&mut self, account: &ActorId, id: &u128, amount: u128) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }
        let prev_balance = self.balance_of(account, id);
        self.set_balance(account, id, prev_balance.saturating_add(amount));
    }

    fn mint_batch(&mut self, account: &ActorId, ids: &[u128], amounts: &[u128]) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }
        ids.iter()
            .enumerate()
            .for_each(|(_s, x)| self.mint(account, x, amounts[_s]));
    }

    fn burn(&mut self, id: &u128, amount: u128) {
        if !self.owner_of(id) {
            panic!("ERC1155: have no ownership of the id")
        }
        let owner_balance = self.balance_of(&msg::source(), id);
        if owner_balance < amount {
            panic!("ERC1155: burn amount exceeds balance")
        }
        self.set_balance(&msg::source(), id, owner_balance.saturating_sub(amount));
    }

    fn burn_batch(&mut self, ids: &[u128], amounts: &[u128]) {
        let owner = &msg::source();

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        if !self.owner_of_batch(ids) {
            panic!("ERC1155: have no ownership of ids")
        }

        for (i, ele) in ids.iter().enumerate() {
            let amount = amounts[i];

            let owner_balance = self.balance_of(owner, ele);

            if owner_balance < amount {
                panic!("ERC1155: burn amount exceeds balance")
            }
            self.set_balance(owner, ele, owner_balance.saturating_sub(amount));
        }
    }

    fn uri(&self, id: u128) -> String {
        self.base_uri.clone().replace("{id}", &format!("{}", id))
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
            StateReply::Balance(ERC1155_TOKEN.balance_of(&account, &id)).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));
    core::mem::forget(encoded);
    result
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Mint(account, id, amount) => {
            ERC1155_TOKEN.mint(&account, &id, amount);
            msg::reply(
                Event::TransferSingle(TransferSingleReply {
                    operator: msg::source(),
                    from: ZERO_ID,
                    to: account,
                    id,
                    amount,
                }),
                0,
            );
        }

        Action::BalanceOf(account, id) => {
            let balance = ERC1155_TOKEN.balance_of(&account, &id);
            msg::reply(Event::Balance(balance), 0);
        }

        Action::BalanceOfBatch(accounts, ids) => {
            let res = ERC1155_TOKEN.balance_of_batch(&accounts, &ids);
            msg::reply(Event::BalanceOfBatch(res), 0);
        }

        Action::MintBatch(account, ids, amounts) => {
            ERC1155_TOKEN.mint_batch(&account, &ids, &amounts);
            msg::reply(
                Event::TransferBatch {
                    operator: msg::source(),
                    from: ZERO_ID,
                    to: account,
                    ids,
                    values: amounts,
                },
                0,
            );
        }

        Action::SafeTransferFrom(from, to, id, amount) => {
            ERC1155_TOKEN.safe_transfer_from(&from, &to, &id, amount);
            msg::reply(
                Event::TransferSingle(TransferSingleReply {
                    operator: msg::source(),
                    from,
                    to,
                    id,
                    amount,
                }),
                0,
            );
        }

        Action::SafeBatchTransferFrom(from, to, ids, amounts) => {
            ERC1155_TOKEN.safe_batch_transfer_from(&from, &to, &ids, &amounts);
            msg::reply(
                Event::TransferBatch {
                    operator: msg::source(),
                    from,
                    to,
                    ids,
                    values: amounts,
                },
                0,
            );
        }

        Action::SetApprovalForAll(operator, approved) => {
            ERC1155_TOKEN.set_approval_for_all(&operator, approved);
            msg::reply(
                Event::ApprovalForAll {
                    owner: msg::source(),
                    operator,
                    approved,
                },
                0,
            );
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
            msg::reply(
                Event::TransferSingle(TransferSingleReply {
                    operator: msg::source(),
                    from: msg::source(),
                    to: ZERO_ID,
                    id,
                    amount,
                }),
                0,
            );
        }

        Action::BurnBatch(ids, amounts) => {
            ERC1155_TOKEN.burn_batch(&ids, &amounts);
            msg::reply(
                Event::TransferBatch {
                    operator: msg::source(),
                    from: msg::source(),
                    to: ZERO_ID,
                    ids,
                    values: amounts,
                },
                0,
            );
        }

        Action::OwnerOf(id) => {
            let res = ERC1155_TOKEN.owner_of(&id);
            msg::reply(res, 0);
        }

        Action::OwnerOfBatch(ids) => {
            let res = ERC1155_TOKEN.owner_of_batch(&ids);
            msg::reply(res, 0);
        }

        Action::URI(id) => {
            let res = ERC1155_TOKEN.uri(id);
            msg::reply(Event::URI(res), 0);
        }
    }
}
