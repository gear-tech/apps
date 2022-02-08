// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.2/contracts/token/ERC1155/IERC1155.sol

#![no_std]
#![feature(const_btree_new)]

#[cfg(test)]
use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

pub mod base;

const GAS_AMOUNT: u64 = 300_000_000;
const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
struct Erc1155Token {
    name: String,
    symbol: String,
    description: String,
    base_uri: String,
    balances: BTreeMap<u128, BTreeMap<ActorId, u128>>,
    operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

static mut ERC1155_TOKEN: Erc1155Token = Erc1155Token {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    description: String::new(),
    balances: BTreeMap::new(),
    operator_approvals: BTreeMap::new(),
};

impl Erc1155Token {
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

    fn balance_of(&self, account: &ActorId, id: &u128) -> u128 {
        self.get_balance(account, id)
    }

    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[u128]) -> Vec<BalanceOfBatchReply> {
        if accounts.len() != ids.len() {
            panic!("ERC1155: accounts and ids length mismatch");
        }

        let mut arr: Vec<BalanceOfBatchReply> = Vec::new();

        for (i, ele) in ids.iter().enumerate() {
            let account = accounts[i];
            let amount = self.get_balance(&account, &ele);

            let obj = BalanceOfBatchReply {
                account: account,
                id: *ele,
                amount: amount,
            };

            arr.push(obj);
        }

        return arr;
    }

    fn mint(&mut self, from: &ActorId, id: &u128, amount: u128) {
        // check owner
        if from == &ZERO_ID {
            panic!("ERC1155: Mint to zero address");
        }
        let old_balance = self.get_balance(from, id);
        self.set_balance(from, id, old_balance.saturating_add(amount));

        // TransferSingle event
    }

    fn mint_batch(&mut self, from: &ActorId, ids: &[u128], amounts: &[u128]) {
        if from == &ZERO_ID {
            panic!("ERC1155: Mint to zero address");
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch");
        }

        for (i, ele) in ids.iter().enumerate() {
            let amount = amounts[i];
            let old_balance = self.get_balance(from, ele);
            self.set_balance(from, ele, old_balance.saturating_add(amount));
        }

        // TransferBatch event
    }

    fn burn_batch(&mut self, from: &ActorId, ids: &[u128], amounts: &[u128]) {
        // TODO
        // check owner
        if from == &ZERO_ID {
            panic!("ERC1155: burn from the zero address");
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (i, ele) in ids.iter().enumerate() {
            let amount = amounts[i];

            let from_balance = self.get_balance(from, ele);

            if from_balance < amount {
                panic!("ERC1155: burn amount exceeds balance")
            }

            self.set_balance(from, ele, from_balance.saturating_sub(amount));
        }

        // TransferBatch event
    }

    fn set_approval_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool) {
        if owner == operator {
            panic!("ERC1155: setting approval status for self")
        }

        self.operator_approvals
            .entry(*owner)
            .or_default()
            .insert(*operator, approved);

        // ApprovalForAll event
    }

    fn is_approved_for_all(&mut self, account: &ActorId, operator: &ActorId) -> &bool {
        self.get_approval(account, operator)
    }

    fn get_approval(&mut self, owner: &ActorId, operator: &ActorId) -> &bool {
        if owner == operator {
            panic!("ERC1155: setting approval status for self")
        }

        self.operator_approvals
            .entry(*owner)
            .or_default()
            .get(operator)
            .unwrap_or(&false)
    }

    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &u128, amount: u128) {
        if from == to {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if *self.get_approval(from, to) {
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

        // TransferSingle event
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[u128],
        amounts: &[u128],
    ) {
        if from == to {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if *self.get_approval(from, to) {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch")
        }

        for (i, ele) in ids.iter().enumerate() {
            let amount = amounts[i];

            let from_balance = self.get_balance(from, ele);

            if from_balance < amount {
                panic!("ERC1155: insufficient balance for transfer")
            }

            self.set_balance(from, ele, from_balance.saturating_sub(amount));
            let to_balance = self.get_balance(to, ele);
            self.set_balance(to, ele, to_balance.saturating_add(amount));
        }

        // TransferBatch event
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum State {
    Name,
    Symbol,
    BalanceOf(ActorId, u128),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    Name(String),
    Symbol(String),
    Balance(u128),
}

gstd::metadata! {
    title: "ERC1155",
    init:
        input: InitConfig,
        // {"name": "GEAR Token", "symbol": "GRT", "base_uri": "baidu.so" }
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
        // TODO
        // url
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
            let transfer_data = TransferSingleReply {
                operator: msg::source(),
                from: ZERO_ID,
                to: account,
                id: id,
                amount: amount,
            };

            msg::reply(Event::TransferSingle(transfer_data), GAS_AMOUNT, 0);
        }
        Action::BalanceOf(account, id) => {
            let balance = ERC1155_TOKEN.balance_of(&account, &id);
            msg::reply(Event::Balance(balance), GAS_AMOUNT, 0);
        }
        Action::BalanceOfBatch(accounts, ids) => {
            let res = ERC1155_TOKEN.balance_of_batch(&accounts, &ids);
            msg::reply(Event::BalanceOfBatch(res), GAS_AMOUNT, 0);
        }
        Action::MintBatch(account, ids, amounts) => {
            ERC1155_TOKEN.mint_batch(&account, &ids, &amounts);

            let payload = Event::TransferBatch {
                operator: msg::source(),
                from: ZERO_ID,
                to: account,
                ids: ids,
                values: amounts,
            };
            msg::reply(payload, GAS_AMOUNT, 0);
        }

        Action::SafeTransferFrom(from, to, id, amount) => {
            ERC1155_TOKEN.safe_transfer_from(&from, &to, &id, amount);

            let transfer_data = TransferSingleReply {
                operator: msg::source(),
                from: from,
                to: to,
                id: id,
                amount: amount,
            };

            msg::reply(Event::TransferSingle(transfer_data), GAS_AMOUNT, 0);
        }

        Action::SafeBatchTransferFrom(from, to, ids, amounts) => {
            ERC1155_TOKEN.safe_batch_transfer_from(&from, &to, &ids, &amounts);

            let payload = Event::TransferBatch {
                operator: msg::source(),
                from: from,
                to: to,
                ids: ids,
                values: amounts,
            };

            msg::reply(payload, GAS_AMOUNT, 0);
        }

        Action::ApproveForAll(owner, operator, approved) => {
            ERC1155_TOKEN.set_approval_for_all(&owner, &operator, approved);

            let payload = Event::ApprovalForAll {
                owner: owner,
                operator: operator,
                approved: approved,
            };

            msg::reply(payload, GAS_AMOUNT, 0);
        }

        Action::IsApprovedForAll(owner, operator) => {
            let approved = ERC1155_TOKEN.is_approved_for_all(&owner, &operator);

            let payload = Event::ApprovalForAll {
                owner: owner,
                operator: operator,
                approved: *approved,
            };

            msg::reply(payload, GAS_AMOUNT, 0);
        }
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Mint(ActorId, u128, u128),
    BalanceOf(ActorId, u128),
    BalanceOfBatch(Vec<ActorId>, Vec<u128>),
    MintBatch(ActorId, Vec<u128>, Vec<u128>),
    SafeTransferFrom(ActorId, ActorId, u128, u128),
    SafeBatchTransferFrom(ActorId, ActorId, Vec<u128>, Vec<u128>),
    ApproveForAll(ActorId, ActorId, bool),
    IsApprovedForAll(ActorId, ActorId),
    // Approve { to: ActorId, id: U256 },
    // Burn(U256),
    // OwnerOf(U256)
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferSingleReply {
    pub operator: ActorId,
    pub from: ActorId,
    pub to: ActorId,
    pub id: u128,
    pub amount: u128,
}
#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct BalanceOfBatchReply {
    pub account: ActorId,
    pub id: u128,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    TransferSingle(TransferSingleReply),
    Balance(u128),
    BalanceOfBatch(Vec<BalanceOfBatchReply>),
    MintOfBatch(Vec<BalanceOfBatchReply>),
    TransferBatch {
        operator: ActorId,
        from: ActorId,
        to: ActorId,
        ids: Vec<u128>,
        values: Vec<u128>,
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
    URI {
        value: String,
        token_id: U256,
    },
}
