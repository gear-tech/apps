#![no_std]

use escrow_io::*;
use ft_io::{FTAction, FTEvent};
use gstd::{
    async_main, exec,
    msg::{self, CodecMessageFuture},
    prelude::*,
    ActorId,
};
use primitive_types::U256;

fn transfer_tokens(
    ft_program_id: ActorId,
    from: ActorId,
    to: ActorId,
    amount: u128,
) -> CodecMessageFuture<FTEvent> {
    msg::send_and_wait_for_reply(ft_program_id, FTAction::Transfer { from, to, amount }, 0).unwrap()
}

fn get(accounts: &mut BTreeMap<AccountId, Account>, account_id: AccountId) -> &mut Account {
    if let Some(account) = accounts.get_mut(&account_id) {
        account
    } else {
        panic!("Account with the {account_id} ID doesn't exist");
    }
}

#[derive(Default)]
struct Escrow {
    ft_program_id: ActorId,
    accounts: BTreeMap<AccountId, Account>,
    id_nonce: U256,
}

impl Escrow {
    /// Creates one escrow account and replies with its ID.
    ///
    /// Requirements:
    /// * `msg::source()` must be a buyer or seller for this account.
    ///
    /// Arguments:
    /// * `buyer`: a buyer.
    /// * `seller`: a seller.
    /// * `amount`: an amount of tokens.
    fn create(&mut self, buyer: ActorId, seller: ActorId, amount: u128) {
        if msg::source() != buyer && msg::source() != seller {
            panic!("msg::source() must be a buyer or seller to create the escrow account");
        }

        let account_id = self.id_nonce;
        self.id_nonce = self.id_nonce.saturating_add(U256::one());

        self.accounts.insert(
            account_id,
            Account {
                buyer,
                seller,
                amount,
                state: AccountState::AwaitingDeposit,
            },
        );

        msg::reply(EscrowEvent::Created(account_id), 0).unwrap();
    }

    /// Makes a deposit from a buyer to an escrow account
    /// and changes an account state to `AwaitingConfirmation`.
    ///
    /// Requirements:
    /// * `msg::source()` must be a buyer saved in the account.
    /// * Account must not be paid or closed.
    ///
    /// Arguments:
    /// * `account_id`: an account ID.
    async fn deposit(&mut self, account_id: AccountId) {
        let account = get(&mut self.accounts, account_id);

        if msg::source() != account.buyer {
            panic!("msg::source() must be a buyer saved in the account to make a deposit");
        }

        if account.state != AccountState::AwaitingDeposit {
            panic!("Paid or closed account can't take a deposit");
        }

        transfer_tokens(
            self.ft_program_id,
            account.buyer,
            exec::program_id(),
            account.amount,
        )
        .await
        .expect("Error when taking a deposit");

        account.state = AccountState::AwaitingConfirmation;

        msg::reply(
            EscrowEvent::Deposited {
                buyer: account.buyer,
                amount: account.amount,
            },
            0,
        )
        .unwrap();
    }

    /// Confirms an escrow account by transferring tokens from it
    /// to a seller and changing an account state to `Closed`.
    ///
    /// Requirements:
    /// * `msg::source()` must be a buyer saved in the account.
    /// * Account must be paid and unclosed.
    ///
    /// Arguments:
    /// * `account_id`: an account ID.
    async fn confirm(&mut self, account_id: AccountId) {
        let account = get(&mut self.accounts, account_id);

        if msg::source() != account.buyer {
            panic!("msg::source() must a buyer saved in the account to confirm it");
        }

        if account.state != AccountState::AwaitingConfirmation {
            panic!("Unpaid or closed account can't be confirmed");
        }

        transfer_tokens(
            self.ft_program_id,
            exec::program_id(),
            account.seller,
            account.amount,
        )
        .await
        .expect("Error when confirming an account");

        account.state = AccountState::Closed;

        msg::reply(
            EscrowEvent::Confirmed {
                amount: account.amount,
                seller: account.seller,
            },
            0,
        )
        .unwrap();
    }

    /// Refunds tokens from an escrow account to a buyer
    /// and changes an account state back to `AwaitingDeposit`
    /// (that is, the account can be reused).
    ///
    /// Requirements:
    /// * `msg::source()` must be a seller saved in the account.
    /// * Account must be paid and unclosed.
    ///
    /// Arguments:
    /// * `account_id`: an account ID.
    async fn refund(&mut self, account_id: AccountId) {
        let account = get(&mut self.accounts, account_id);

        if msg::source() != account.seller {
            panic!("msg::source() must be a seller saved in the account to refund");
        }

        if account.state != AccountState::AwaitingConfirmation {
            panic!("Unpaid or closed account can't be refunded");
        }

        transfer_tokens(
            self.ft_program_id,
            exec::program_id(),
            account.buyer,
            account.amount,
        )
        .await
        .expect("Error when refunding from an account");

        account.state = AccountState::AwaitingDeposit;

        msg::reply(
            EscrowEvent::Refunded {
                amount: account.amount,
                buyer: account.buyer,
            },
            0,
        )
        .unwrap();
    }

    /// Cancels (early closes) an escrow account by changing its state to `Closed`.
    ///
    /// Requirements:
    /// * `msg::source()` must be a buyer or seller saved in the account.
    /// * Account must not be paid or closed.
    ///
    /// Arguments:
    /// * `account_id`: an account ID.
    async fn cancel(&mut self, account_id: AccountId) {
        let account = get(&mut self.accounts, account_id);

        if msg::source() != account.buyer && msg::source() != account.seller {
            panic!("msg::source() must be a buyer or seller saved in the account to cancel it");
        }

        if account.state != AccountState::AwaitingDeposit {
            panic!("Paid or closed account can't be canceled");
        }

        account.state = AccountState::Closed;

        msg::reply(
            EscrowEvent::Cancelled {
                buyer: account.buyer,
                seller: account.seller,
                amount: account.amount,
            },
            0,
        )
        .unwrap();
    }
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
pub extern "C" fn init() {
    let config: InitEscrow = msg::load().expect("Unable to decode InitEscrow");
    let escrow = Escrow {
        ft_program_id: config.ft_program_id,
        ..Default::default()
    };
    unsafe {
        ESCROW = Some(escrow);
    }
}

#[async_main]
pub async fn main() {
    let action: EscrowAction = msg::load().expect("Unable to decode EscrowAction");
    let escrow = unsafe { ESCROW.get_or_insert(Default::default()) };
    match action {
        EscrowAction::Create {
            buyer,
            seller,
            amount,
        } => escrow.create(buyer, seller, amount),
        EscrowAction::Deposit(account_id) => escrow.deposit(account_id).await,
        EscrowAction::Confirm(account_id) => escrow.confirm(account_id).await,
        EscrowAction::Refund(account_id) => escrow.refund(account_id).await,
        EscrowAction::Cancel(account_id) => escrow.cancel(account_id).await,
    }
}

#[no_mangle]
pub extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: EscrowState = msg::load().expect("Unable to decode EscrowState");
    let escrow = unsafe { ESCROW.get_or_insert(Default::default()) };
    let encoded = match state {
        EscrowState::GetInfo(account_id) => {
            EscrowStateReply::Info(*get(&mut escrow.accounts, account_id)).encode()
        }
    };
    gstd::util::to_leak_ptr(encoded)
}

gstd::metadata! {
    title: "Escrow",

    init:
        input: InitEscrow,
    handle:
        input: EscrowAction,
        output: EscrowEvent,
    state:
        input: EscrowState,
        output: EscrowStateReply,
}
