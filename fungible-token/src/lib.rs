// Fungible Token Smart Contract.
// Implementation based on https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/ERC20.sol

#![no_std]
#![feature(const_btree_new)]

use fungible_token_messages::{
    Action, ApproveReply, Event, InitConfig, TransferFromReply, TransferReply,
};
use gstd::{msg, prelude::*, ActorId};

const GAS_AMOUNT: u64 = 300_000_000;

#[derive(Debug)]
struct FungibleToken {
    /// Name of the token.
    name: String,
    /// Symbol of the token.
    symbol: String,
    /// Total supply of the token.
    total_supply: u128,
    /// Map to hold balances of token holders.
    balances: BTreeMap<ActorId, u128>,
    /// Map to hold allowance information of token holders.
    allowances: BTreeMap<ActorId, BTreeMap<ActorId, u128>>,
    /// Creator of the token.
    creator: ActorId,
    /// Creator approved set of admins, who can do mint/burn.
    admins: BTreeSet<ActorId>,
}

static mut FUNGIBLE_TOKEN: FungibleToken = FungibleToken {
    name: String::new(),
    symbol: String::new(),
    total_supply: 0,
    balances: BTreeMap::new(),
    allowances: BTreeMap::new(),
    creator: ActorId::new([0u8; 32]),
    admins: BTreeSet::new(),
};

impl FungibleToken {
    /// Token creator account add `account` as admin to token.
    fn add_admin(&mut self, account: &ActorId) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source {
                panic!("FungibleToken: Only token creator can add admin.");
            }
            if *account != FUNGIBLE_TOKEN.creator {
                self.admins.insert(*account);
            }
        }
    }
    /// Token creator account remove `account` as admin from token.
    fn remove_admin(&mut self, account: &ActorId) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source {
                panic!("FungibleToken: Only token creator can remove admin.");
            }
            if *account != FUNGIBLE_TOKEN.creator {
                self.admins.remove(account);
            }
        }
    }
    fn increase_total_supply(&mut self, amount: u128) {
        self.total_supply = self.total_supply.saturating_add(amount);
    }
    fn decrease_total_supply(&mut self, amount: u128) {
        self.total_supply = self.total_supply.saturating_sub(amount);
    }
    fn set_balance(&mut self, account: &ActorId, amount: u128) {
        self.balances.insert(*account, amount);
    }
    fn get_balance(&self, account: &ActorId) -> u128 {
        *self.balances.get(account).unwrap_or(&0)
    }
    /// Executed on receiving `fungible-token-message::BalanceOf`, returns token balance of `account`.
    fn balance_of(&self, account: &ActorId) -> u128 {
        self.get_balance(account)
    }
    /// Executed on receiving `fungible-token-messages::MintInput`.
    /// If executed by creator or admins of the token then mints `amount` tokens into `account`.
    fn mint(&mut self, account: &ActorId, amount: u128) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source && !FUNGIBLE_TOKEN.admins.contains(&source) {
                panic!("FungibleToken: Only token creator or designated admins can mint tokens.");
            }
        }
        let zero = ActorId::new([0u8; 32]);
        if account == &zero {
            panic!("FungibleToken: Mint to zero address.");
        }
        unsafe {
            self.increase_total_supply(amount);
            let old_balance = FUNGIBLE_TOKEN.get_balance(account);
            self.set_balance(account, old_balance.saturating_add(amount));
        }
    }
    /// Executed on receiving `fungible-token-messages::BurnInput`.
    /// If executed by creator or admins of the token then burns `amount` tokens from `account`.
    fn burn(&mut self, account: &ActorId, amount: u128) {
        unsafe {
            let source = msg::source();
            if FUNGIBLE_TOKEN.creator != source && !FUNGIBLE_TOKEN.admins.contains(&source) {
                panic!("FungibleToken: Only token creator or designated admins can burn tokens.");
            }
        }
        let zero = ActorId::new([0u8; 32]);
        if account == &zero {
            panic!("FungibleToken: Burn from zero address.");
        }
        unsafe {
            self.decrease_total_supply(amount);
            let old_balance = FUNGIBLE_TOKEN.get_balance(account);
            self.set_balance(account, old_balance.saturating_sub(amount));
        }
    }
    /// Executed on receiving `fungible-token-messages::TransferInput` or `fungible-token-messages::TransferFromInput`.
    /// Transfers `amount` tokens from `sender` account to `recipient` account.
    fn transfer(&mut self, sender: &ActorId, recipient: &ActorId, amount: u128) {
        let zero = ActorId::new([0u8; 32]);
        if sender == &zero {
            panic!("FungibleToken: Transfer from zero address.");
        }
        if recipient == &zero {
            panic!("FungibleToken: Transfer to zero address.");
        }
        let sender_balance = self.get_balance(sender);
        if amount > sender_balance {
            panic!(
                "FungibleToken: Transfer amount {:?} exceeds sender {:?} balance {:?}.",
                amount, sender, sender_balance
            );
        }
        self.set_balance(sender, sender_balance.saturating_sub(amount));
        let recipient_balance = self.get_balance(recipient);
        self.set_balance(recipient, recipient_balance.saturating_add(amount));
    }
    /// Executed on receiving `fungible-token-messages::ApproveInput`.
    /// Adds/Updates allowance entry for `spender` account to tranfer upto `amount` from `owner` account.
    fn approve(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let zero = ActorId::new([0u8; 32]);
        if spender == &zero {
            panic!("FungibleToken: Approve to zero address.");
        }
        self.allowances
            .entry(*owner)
            .or_default()
            .insert(*spender, amount);
    }
    /// To find maximum value allowed to be transfer by `spender` from `owner` account.
    fn get_allowance(&self, owner: &ActorId, spender: &ActorId) -> u128 {
        *self
            .allowances
            .get(owner)
            .and_then(|m| m.get(spender))
            .unwrap_or(&0)
    }
    /// To increase allowance of `spender` for `owner` account.
    fn increase_allowance(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let allowance = self.get_allowance(owner, spender);
        self.approve(owner, spender, allowance.saturating_add(amount));
    }
    /// To decrease allowance of `spender` for `owner` account.
    fn decrease_allowance(&mut self, owner: &ActorId, spender: &ActorId, amount: u128) {
        let allowance = self.get_allowance(owner, spender);
        if amount > allowance {
            panic!("FungibleToken: Decreased allowance below zero.");
        }
        self.approve(owner, spender, allowance - amount);
    }
    /// Transfer `amount` from `owner` account to `recipient` account if `sender`'s allowance permits.
    fn transfer_from(
        &mut self,
        owner: &ActorId,
        sender: &ActorId,
        recipient: &ActorId,
        amount: u128,
    ) -> u128 {
        let current_allowance = self.get_allowance(owner, sender);
        if current_allowance < amount {
            panic!("FungibleToken: Transfer amount exceeds allowance");
        }
        self.transfer(owner, recipient, amount);
        let new_limit = current_allowance - amount;
        self.approve(owner, sender, new_limit);
        new_limit
    }
}

gstd::metadata! {
    title: "FungibleToken",
    init:
        input: InitConfig,
    handle:
        input: Action,
        output: Event,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");

    match action {
        Action::Mint(mint_input) => {
            FUNGIBLE_TOKEN.mint(&mint_input.account, mint_input.amount);
            let zero = ActorId::new([0u8; 32]);
            let transfer_data = TransferReply {
                from: zero,
                to: mint_input.account,
                amount: mint_input.amount,
            };
            msg::reply(Event::Transfer(transfer_data), GAS_AMOUNT, 0);
        }
        Action::Burn(burn_input) => {
            FUNGIBLE_TOKEN.burn(&burn_input.account, burn_input.amount);
            let zero = ActorId::new([0u8; 32]);
            let transfer_data = TransferReply {
                from: burn_input.account,
                to: zero,
                amount: burn_input.amount,
            };
            msg::reply(Event::Transfer(transfer_data), GAS_AMOUNT, 0);
        }
        Action::Transfer(transfer_data) => {
            let from = msg::source();
            let to = transfer_data.to;
            let amount = transfer_data.amount;
            FUNGIBLE_TOKEN.transfer(&from, &to, amount);
            let transfer_data = TransferReply { from, to, amount };
            msg::reply(Event::Transfer(transfer_data), GAS_AMOUNT, 0);
        }
        Action::Approve(approve_data) => {
            let owner = msg::source();
            let spender = approve_data.spender;
            let amount = approve_data.amount;
            FUNGIBLE_TOKEN.approve(&owner, &spender, amount);
            let approve_data = ApproveReply {
                owner,
                spender,
                amount,
            };
            msg::reply(Event::Approval(approve_data), GAS_AMOUNT, 0);
        }
        Action::TransferFrom(transfer_data) => {
            let owner = transfer_data.owner;
            let sender = msg::source();
            let recipient = transfer_data.to;
            let amount = transfer_data.amount;
            let new_limit = FUNGIBLE_TOKEN.transfer_from(&owner, &sender, &recipient, amount);
            let tranfer_from = TransferFromReply {
                owner,
                sender,
                recipient,
                amount,
                new_limit,
            };
            msg::reply(Event::TransferFrom(tranfer_from), GAS_AMOUNT, 0);
        }
        Action::IncreaseAllowance(approve_data) => {
            let owner = msg::source();
            let spender = approve_data.spender;
            let amount = approve_data.amount;
            FUNGIBLE_TOKEN.increase_allowance(&owner, &spender, amount);
            let approve_data = ApproveReply {
                owner,
                spender,
                amount,
            };
            msg::reply(Event::Approval(approve_data), GAS_AMOUNT, 0);
        }
        Action::DecreaseAllowance(approve_data) => {
            let owner = msg::source();
            let spender = approve_data.spender;
            let amount = approve_data.amount;
            FUNGIBLE_TOKEN.decrease_allowance(&owner, &spender, amount);
            let approve_data = ApproveReply {
                owner,
                spender,
                amount,
            };
            msg::reply(Event::Approval(approve_data), GAS_AMOUNT, 0);
        }
        Action::TotalIssuance => {
            let total_supply = FUNGIBLE_TOKEN.total_supply;
            msg::reply(Event::TotalIssuance(total_supply), GAS_AMOUNT, 0);
        }
        Action::BalanceOf(account) => {
            let balance = FUNGIBLE_TOKEN.balance_of(&account);
            msg::reply(Event::Balance(balance), GAS_AMOUNT, 0);
        }
        Action::AddAdmin(account) => {
            FUNGIBLE_TOKEN.add_admin(&account);
            msg::reply(Event::AdminAdded(account), GAS_AMOUNT, 0);
        }
        Action::RemoveAdmin(account) => {
            FUNGIBLE_TOKEN.remove_admin(&account);
            msg::reply(Event::AdminRemoved(account), GAS_AMOUNT, 0);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    FUNGIBLE_TOKEN.name = config.name;
    FUNGIBLE_TOKEN.symbol = config.symbol;
    FUNGIBLE_TOKEN.creator = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn handle_reply() {}
