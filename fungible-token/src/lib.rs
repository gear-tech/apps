#![no_std]

#[cfg(test)]
mod tests;

use ft_io::*;
use gstd::{exec, debug, msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug, Default)]
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
}

static mut FUNGIBLE_TOKEN: Option<FungibleToken> = None;

impl FungibleToken {
    /// Executed on receiving `fungible-token-messages::MintInput`.
    fn mint(&mut self, amount: u128) {
        debug!("msg::source() {:?}", msg::source());
        let balance = self.balances.get(&msg::source()).unwrap_or(&0);
        debug!("balance before {:?}", balance);
        self.balances
            .entry(msg::source())
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
        self.total_supply += amount;
        let balance = self.balances.get(&msg::source()).unwrap_or(&0);
        debug!("balance after {:?}", balance);
        msg::reply(
            Event::Transfer {
                from: ZERO_ID,
                to: msg::source(),
                amount,
            },
            0,
        );
    }
    /// Executed on receiving `fungible-token-messages::BurnInput`.
    fn burn(&mut self, amount: u128) {
        if self.balances.get(&msg::source()).unwrap_or(&0) < &amount {
            panic!("Amount exceeds account balance");
        }
        self.balances
            .entry(msg::source())
            .and_modify(|balance| *balance -= amount);
        self.total_supply -= amount;

        msg::reply(
            Event::Transfer {
                from: msg::source(),
                to: ZERO_ID,
                amount,
            },
            0,
        );
    }
    /// Executed on receiving `fungible-token-messages::TransferInput` or `fungible-token-messages::TransferFromInput`.
    /// Transfers `amount` tokens from `sender` account to `recipient` account.
    fn transfer(&mut self, from: &ActorId, to: &ActorId, amount: u128) {
        if from == &ZERO_ID || to == &ZERO_ID {
            panic!("Zero addresses");
        };
        if !self.can_transfer(from, amount) {
            panic!("Not allowed to transfer")
        }
        if self.balances.get(from).unwrap_or(&0) < &amount {
            panic!("Amount exceeds account balance");
        }
        self.balances
            .entry(*from)
            .and_modify(|balance| *balance -= amount);
        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
        msg::reply(
            Event::Transfer {
                from: *from,
                to: *to,
                amount,
            },
            0,
        );
    }

    /// Executed on receiving `fungible-token-messages::ApproveInput`.
    fn approve(&mut self, to: &ActorId, amount: u128) {
        if to == &ZERO_ID {
            panic!("Approve to zero address");
        }
        self.allowances
            .entry(exec::origin())
            .or_default()
            .insert(*to, amount);
        msg::reply(
            Event::Approve {
                from: exec::origin(),
                to: *to,
                amount,
            },
            0,
        );
    }

    fn can_transfer(&mut self, from: &ActorId, amount: u128) -> bool {
        if from == &msg::source()
            || from == &exec::origin()
            || self.balances.get(&msg::source()).unwrap_or(&0) >= &amount
            || self.balances.get(&exec::origin()).unwrap_or(&0) >= &amount
        {
            return true;
        }
        if let Some(allowed_amount) = self
            .allowances
            .get(from)
            .and_then(|m| m.get(&msg::source()))
        {
            if allowed_amount >= &amount {
                self.allowances.entry(*from).and_modify(|m| {
                    m.entry(msg::source()).and_modify(|a| *a -= amount);
                });
                return true;
            }
        }
        false
    }
}

gstd::metadata! {
    title: "FungibleToken",
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
    let ft: &mut FungibleToken = FUNGIBLE_TOKEN.get_or_insert(FungibleToken::default());
    match action {
        Action::Mint(amount) => {
            ft.mint(amount);
        }
        Action::Burn(amount) => {
            ft.burn(amount);
        }
        Action::Transfer { from, to, amount } => {
            ft.transfer(&from, &to, amount);
        }
        Action::Approve { to, amount } => {
            ft.approve(&to, amount);
        }
        Action::TotalSupply => {
            msg::reply(Event::TotalSupply(ft.total_supply), 0);
        }
        Action::BalanceOf(account) => {
            let balance = ft.balances.get(&account).unwrap_or(&0);
            msg::reply(Event::Balance(*balance), 0);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    let ft = FungibleToken {
        name: config.name,
        symbol: config.symbol,
        ..FungibleToken::default()
    };
    FUNGIBLE_TOKEN = Some(ft);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let ft: &mut FungibleToken = FUNGIBLE_TOKEN.get_or_insert(FungibleToken::default());
    let encoded = match query {
        State::Name => StateReply::Name(ft.name.clone()).encode(),
        State::Symbol => StateReply::Name(ft.symbol.clone()).encode(),
        State::Decimals => StateReply::Decimals(18).encode(),
        State::TotalSupply => StateReply::TotalSupply(ft.total_supply).encode(),
        State::BalanceOf(account) => {
            let balance = ft.balances.get(&account).unwrap_or(&0);
            StateReply::Balance(*balance).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));
    core::mem::forget(encoded);
    result
}

#[no_mangle]
pub unsafe extern "C" fn handle_reply() {}
