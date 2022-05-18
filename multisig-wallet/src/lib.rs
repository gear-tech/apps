#![no_std]
#![feature(const_btree_new)]

extern crate alloc;

use core::cmp::min;
use gstd::{exec, msg, prelude::*, ActorId};
pub use multisig_wallet_io::*;
use primitive_types::U256;
pub mod state;
use state::*;

const MAX_OWNERS_COUNT: u64 = 50;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

pub struct Transaction {
    destination: ActorId,
    payload: Vec<u8>,
    value: u128,
    executed: bool,
}

#[derive(Default)]
pub struct MultisigWallet {
    pub transactions: BTreeMap<U256, Transaction>,
    pub confirmations: BTreeMap<U256, BTreeMap<ActorId, bool>>,
    pub is_owner: BTreeMap<ActorId, bool>,
    pub owners: Vec<ActorId>,
    pub required: u64,
    pub transaction_count: U256,
}

static mut WALLET: Option<MultisigWallet> = None;

fn validate_requirement(owners_count: usize, required: u64) {
    if owners_count > MAX_OWNERS_COUNT.try_into().unwrap() {
        panic!("Too much owners");
    }

    if (owners_count as u64) < required {
        panic!("Required count more than owners count");
    }

    if required < 1 {
        panic!("Required quantity must be greater than zero");
    }
}

fn validate_not_null_address(actor_id: &ActorId) {
    if *actor_id == ZERO_ID {
        panic!("actor_id can not be zero");
    }
}

impl MultisigWallet {
    fn validate_only_wallet(&self) {
        if msg::source() != exec::program_id() {
            panic!("Only wallet can call it")
        }
    }

    fn validate_owner_doesnt_exists(&self, owner: &ActorId) {
        if self.is_owner.get(owner).copied().unwrap_or(false) {
            panic!("Owner already exists")
        }
    }

    fn validate_owner_exists(&self, owner: &ActorId) {
        if !self.is_owner.get(owner).copied().unwrap_or(false) {
            panic!("Owner doesn't exists")
        }
    }

    fn validate_transaction_exists(&self, transaction_id: &U256) {
        if !self.transactions.contains_key(transaction_id) {
            panic!("Transaction with this ID doesn't exists")
        }
    }

    fn validate_confirmed(&self, transaction_id: &U256, owner: &ActorId) {
        if !self
            .confirmations
            .get(transaction_id)
            .and_then(|confirmations| confirmations.get(owner))
            .copied()
            .unwrap_or(false)
        {
            panic!("There is no confirmation of this owner")
        }
    }

    fn validate_not_confirmed(&self, transaction_id: &U256, owner: &ActorId) {
        if self
            .confirmations
            .get(transaction_id)
            .and_then(|confirmations| confirmations.get(owner))
            .copied()
            .unwrap_or(false)
        {
            panic!("There is confirmation of this owner")
        }
    }

    fn validate_not_executed(&self, transaction_id: &U256) {
        if self
            .transactions
            .get(transaction_id)
            .map(|t| t.executed)
            .unwrap_or(false)
        {
            panic!("Transaction has been already executed")
        }
    }

    /// Allows to add a new owner. Transaction has to be sent by wallet.
    /// owner - Address of new owner.
    fn add_owner(&mut self, owner: &ActorId) {
        self.validate_only_wallet();
        self.validate_owner_doesnt_exists(owner);
        validate_requirement(self.owners.len() + 1, self.required);

        *self.is_owner.entry(*owner).or_insert(true) = true;
        self.owners.push(*owner);

        msg::reply(MWEvent::OwnerAddition { owner: *owner }, 0).unwrap();
    }

    /// Allows to remove an owner. Transaction has to be sent by wallet.
    /// owner Address of owner.
    fn remove_owner(&mut self, owner: &ActorId) {
        self.validate_only_wallet();
        self.validate_owner_exists(owner);
        let next_owners_count = self.owners.len() - 1;
        validate_requirement(
            next_owners_count,
            min(next_owners_count as u64, self.required),
        );

        *self.is_owner.get_mut(owner).unwrap() = false;
        self.owners.retain(|&x| x != *owner);

        if (self.owners.len() as u64) < self.required {
            self.change_requirement(self.owners.len().try_into().unwrap());
        }

        msg::reply(MWEvent::OwnerRemoval { owner: *owner }, 0).unwrap();
    }

    /// Allows to replace an owner with a new owner. Transaction has to be sent by wallet.
    /// owner Address of owner to be replaced.
    /// newOwner Address of new owner.
    fn replace_owner(&mut self, old_owner: &ActorId, new_owner: &ActorId) {
        self.validate_only_wallet();
        self.validate_owner_exists(old_owner);
        self.validate_owner_doesnt_exists(new_owner);

        let old_owner_index = self
            .owners
            .iter()
            .position(|x| *x == *old_owner)
            .expect("Can't find old owner");
        self.owners[old_owner_index] = *new_owner;

        *self.is_owner.entry(*old_owner).or_default() = false;
        *self.is_owner.entry(*new_owner).or_default() = true;

        msg::reply(
            MWEvent::OwnerReplace {
                old_owner: *old_owner,
                new_owner: *new_owner,
            },
            0,
        )
        .unwrap();
    }

    /// Allows to change the number of required confirmations. Transaction has to be sent by wallet.
    /// required Number of required confirmations.
    fn change_requirement(&mut self, required: u64) {
        self.validate_only_wallet();
        validate_requirement(self.owners.len(), required);

        self.required = required;

        msg::reply(MWEvent::RequirementChange(required.into()), 0).unwrap();
    }

    ///  Allows an owner to submit and confirm a transaction.
    ///  destination Transaction target address.
    ///  value Transaction ether value.
    ///  data Transaction data payload.
    ///  Returns transaction ID.
    fn submit_transaction(&mut self, destination: &ActorId, data: Vec<u8>, value: u128) -> U256 {
        let transaction_id = self.add_transaction(destination, data, value);
        self.confirm_transaction(&transaction_id);

        msg::reply(MWEvent::Submission { transaction_id }, 0).unwrap();

        transaction_id
    }

    /// Allows an owner to confirm a transaction.
    /// transactionId Transaction ID.
    fn confirm_transaction(&mut self, transaction_id: &U256) {
        self.validate_owner_exists(&msg::source());
        self.validate_transaction_exists(transaction_id);
        self.validate_not_confirmed(transaction_id, &msg::source());

        let confirmation = self
            .confirmations
            .entry(*transaction_id)
            .or_insert(BTreeMap::new())
            .entry(msg::source())
            .or_insert(true);

        *confirmation = true;

        self.execute_transaction(transaction_id, None::<fn()>);
    }

    fn external_confirm_transaction(&mut self, transaction_id: &U256) {
        self.confirm_transaction(transaction_id);

        msg::reply(
            MWEvent::Confirmation {
                sender: msg::source(),
                transaction_id: *transaction_id,
            },
            0,
        )
        .unwrap();
    }

    /// Allows an owner to revoke a confirmation for a transaction.
    /// transactionId Transaction ID.
    fn revoke_confirmation(&mut self, transaction_id: &U256) {
        self.validate_owner_exists(&msg::source());
        self.validate_confirmed(transaction_id, &msg::source());
        self.validate_not_executed(transaction_id);

        let confirmation = self
            .confirmations
            .entry(*transaction_id)
            .or_insert(BTreeMap::new())
            .entry(msg::source())
            .or_insert(false);

        *confirmation = false;

        msg::reply(
            MWEvent::Revocation {
                sender: msg::source(),
                transaction_id: *transaction_id,
            },
            0,
        )
        .unwrap();
    }

    /// Allows anyone to execute a confirmed transaction.
    /// transactionId Transaction ID.
    fn execute_transaction<F>(&mut self, transaction_id: &U256, completion: Option<F>)
    where
        F: Fn(),
    {
        let sender = msg::source();
        self.validate_owner_exists(&sender);
        self.validate_confirmed(transaction_id, &sender);
        self.validate_not_executed(transaction_id);

        if !self.is_confirmed(transaction_id) {
            return;
        }

        let txn = self.transactions.get_mut(transaction_id).unwrap();

        msg::send_bytes(txn.destination, txn.payload.clone(), txn.value)
            .expect("Sending message failed");

        txn.executed = true;

        if let Some(completion) = completion {
            completion();
        }
    }

    fn external_execute_transaction(&mut self, transaction_id: &U256) {
        let completion = || {
            let payload = MWEvent::Execution {
                transaction_id: *transaction_id,
            };

            msg::reply(payload, 0).unwrap();
        };

        self.execute_transaction(transaction_id, Some(completion));
    }

    /*
     * Internal functions
     */

    /// Returns the confirmation status of a transaction.
    /// transactionId Transaction ID.
    fn is_confirmed(&self, transaction_id: &U256) -> bool {
        let count = self.get_confirmation_count(transaction_id);

        count >= self.required
    }

    /// Adds a new transaction to the transaction mapping, if transaction does not exist yet.
    /// destination Transaction target address.
    /// value Transaction ether value.
    /// data Transaction data payload.
    /// Returns transaction ID.
    fn add_transaction(&mut self, destination: &ActorId, data: Vec<u8>, value: u128) -> U256 {
        validate_not_null_address(destination);
        let transaction_id = self.transaction_count;
        let transaction = Transaction {
            destination: *destination,
            payload: data,
            value,
            executed: false,
        };

        self.transactions.insert(transaction_id, transaction);
        self.transaction_count += 1.into();

        transaction_id
    }

    /*
     * State
     */

    /// Returns number of confirmations of a transaction.
    /// transactionId Transaction ID.
    /// Number of confirmations.
    fn get_confirmation_count(&self, transaction_id: &U256) -> u64 {
        self.owners
            .iter()
            .map(|owner| {
                self.confirmations
                    .get(transaction_id)
                    .and_then(|confirmations| confirmations.get(owner))
                    .copied()
                    .unwrap_or(false)
            })
            .filter(|confirm| *confirm)
            .count()
            .try_into()
            .unwrap()
    }

    /// Returns total number of transactions after filers are applied.
    /// pending Include pending transactions.
    /// executed Include executed transactions.
    /// Total number of transactions after filters are applied.
    fn get_transaction_count(&self, pending: bool, executed: bool) -> u64 {
        self.transactions
            .values()
            .filter(|transaction| {
                (pending && !transaction.executed) || (executed && transaction.executed)
            })
            .count()
            .try_into()
            .unwrap()
    }

    /// Returns list of owners.
    /// List of owner addresses.
    fn get_owners(&self) -> Vec<ActorId> {
        self.owners.clone()
    }

    /// Returns array with owner addresses, which confirmed transaction.
    /// transactionId Transaction ID.
    /// Returns array of owner addresses.
    fn get_confirmations(&self, transaction_id: &U256) -> Vec<ActorId> {
        self.confirmations
            .get(transaction_id)
            .expect("There is no transaction with this ID")
            .iter()
            .filter(|(_, confirmed)| **confirmed)
            .map(|(actor, _)| *actor)
            .collect()
    }

    /// Returns list of transaction IDs in defined range.
    /// from Index start position of transaction array.
    /// to Index end position of transaction array(not included).
    /// pending Include pending transactions.
    /// executed Include executed transactions.
    /// Returns array of transaction IDs.
    fn get_transaction_ids(&self, from: u64, to: u64, pending: bool, executed: bool) -> Vec<U256> {
        self.transactions
            .iter()
            .filter(|(_, txn)| (pending && !txn.executed) || (executed && txn.executed))
            .map(|(id, _)| *id)
            .take(to.try_into().unwrap())
            .skip(from.try_into().unwrap())
            .collect()
    }
}

gstd::metadata! {
    title: "MultisigWallet",
    init:
        input : MWInitConfig,
    handle:
        input : MWAction,
        output : MWEvent,
    state:
        input: State,
        output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: MWInitConfig = msg::load().expect("Unable to decode InitDao");

    let owners_count = config.owners.len();

    validate_requirement(owners_count, config.required);

    let mut wallet = MultisigWallet::default();

    for owner in &config.owners {
        if wallet.is_owner.contains_key(owner) {
            panic!("The same owner contained twice")
        } else {
            wallet.is_owner.insert(*owner, true);
        }
    }

    wallet.required = config.required;
    wallet.owners = config.owners;

    WALLET = Some(wallet);
}

#[gstd::async_main]
async unsafe fn main() {
    let action = match msg::load::<MWAction>() {
        Ok(action) => action,
        Err(_) => {
            let bytes: Vec<u8> = msg::load_bytes();
            MWAction::decode(&mut bytes.as_slice()).expect("Could not load Action")
        }
    };

    let wallet: &mut MultisigWallet = unsafe { WALLET.get_or_insert(MultisigWallet::default()) };
    match action {
        MWAction::AddOwner(owner) => wallet.add_owner(&owner),
        MWAction::RemoveOwner(owner) => wallet.remove_owner(&owner),
        MWAction::ReplaceOwner {
            old_owner,
            new_owner,
        } => wallet.replace_owner(&old_owner, &new_owner),
        MWAction::ChangeRequiredConfirmationsCount(count) => wallet.change_requirement(count),
        MWAction::SubmitTransaction {
            destination,
            data,
            value,
        } => {
            wallet.submit_transaction(&destination, data, value);
        }
        MWAction::ConfirmTransaction(transaction_id) => {
            wallet.external_confirm_transaction(&transaction_id)
        }
        MWAction::RevokeConfirmation(transaction_id) => wallet.revoke_confirmation(&transaction_id),
        MWAction::ExecuteTransaction(transaction_id) => {
            wallet.external_execute_transaction(&transaction_id)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: State = msg::load().expect("failed to decode input argument");
    let wallet: &mut MultisigWallet = WALLET.get_or_insert(MultisigWallet::default());
    let encoded = match state {
        State::ConfirmationsCount(transaction_id) => {
            StateReply::ConfirmationCount(wallet.get_confirmation_count(&transaction_id))
        }
        State::TransactionsCount { pending, executed } => {
            StateReply::TransactionsCount(wallet.get_transaction_count(pending, executed))
        }
        State::Owners => StateReply::Owners(wallet.get_owners()),
        State::Confirmations(transaction_id) => {
            StateReply::Confirmations(wallet.get_confirmations(&transaction_id))
        }
        State::TransactionIds {
            from_index,
            to_index,
            pending,
            executed,
        } => StateReply::TransactionIds(
            wallet.get_transaction_ids(from_index, to_index, pending, executed),
        ),
        State::IsConfirmed(transaction_id) => {
            StateReply::IsConfirmed(wallet.is_confirmed(&transaction_id))
        }
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}
