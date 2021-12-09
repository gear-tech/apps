<<<<<<< HEAD
=======
// Non-Fungible Token Smart Contract.
// Implementation based on https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/ERC721.sol?

>>>>>>> Add non-fungible token program
#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
<<<<<<< HEAD
use gstd::{exec, msg, prelude::*, ActorId};
pub mod base;
use base::NonFungibleTokenBase;
pub mod token;
use token::TokenMetadata;

=======
use gstd::{debug, exec, msg, prelude::*, ActorId};
>>>>>>> Add non-fungible token program
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

const GAS_RESERVE: u64 = 500_000_000;
<<<<<<< HEAD
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());

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
=======

#[derive(Debug)]
struct NonFungibleToken {
    name: String,
    symbol: String,
    base_uri: String,
    token_id: U256,
    token_owner: BTreeMap<U256, ActorId>,
    token_approvals: BTreeMap<U256, ActorId>,
    owned_tokens_count: BTreeMap<ActorId, U256>,
    operator_approval: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

static mut NON_FUNGIBLE_TOKEN: NonFungibleToken = NonFungibleToken {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    token_id: U256::zero(),
    token_owner: BTreeMap::new(),
    token_approvals: BTreeMap::new(),
    owned_tokens_count: BTreeMap::new(),
    operator_approval: BTreeMap::new(),
};

impl NonFungibleToken {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn set_base_uri(&mut self, base_uri: String) {
        self.base_uri = base_uri;
    }

    fn base_uri(&self) -> &str {
        &self.base_uri
    }

    fn exists(&self, token_id: U256) -> bool {
        self.token_owner.contains_key(&token_id)
    }

    fn is_token_owner(&self, token_id: U256, account: &ActorId) -> bool {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        account == self.token_owner.get(&token_id).unwrap_or(&zero)
    }

    fn is_authorized_source(&self, token_id: U256, account: &ActorId) -> bool {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());

        let owner = self.token_owner.get(&token_id).unwrap_or(&zero);

        if owner == account {
            return true;
        }

        if self.token_approvals.get(&token_id).unwrap() == account {
            return true;
        }

        if *self
            .operator_approval
            .get(owner)
            .unwrap()
            .get(account)
            .unwrap()
        {
            return true;
        }

        return false;
    }

    fn mint(&mut self, account: &ActorId) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("NonFungibleToken: Mint to zero address.");
        }

        self.token_owner.insert(self.token_id, *account);

        let zero = U256::zero();
        let balance = *self.owned_tokens_count.get(account).unwrap_or(&zero);
        self.owned_tokens_count
            .insert(*account, balance.saturating_add(U256::one()));

        let transfer_token = Transfer {
            from: H256::zero(),
            to: H256::from_slice(account.as_ref()),
            token_id: self.token_id,
        };

        self.token_id = self.token_id.saturating_add(U256::one());

        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn burn(&mut self, account: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        if account == &zero {
            panic!("NonFungibleToken: Burn from zero address.");
        }
        if !self.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.is_token_owner(token_id, account) {
            panic!("NonFungibleToken: account is not owner");
        }

        self.token_approvals.remove(&token_id);
        self.token_owner.remove(&token_id);
        let balance = *self
            .owned_tokens_count
            .get(account)
            .unwrap_or(&U256::zero());
        self.owned_tokens_count
            .insert(*account, balance.saturating_sub(U256::one()));

        let transfer_token = Transfer {
            from: H256::from_slice(account.as_ref()),
            to: H256::zero(),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn token_uri(&self, token_id: U256) {
        let token_uri = format!("{}/{}", self.base_uri, token_id.to_string());

        msg::reply(
            Event::TokenURI(token_uri),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn transfer(&mut self, from: &ActorId, to: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());

        if from == &zero {
            panic!("NonFungibleToken: Transfer from zero address.");
        }

        if to == &zero {
            panic!("NonFungibleToken: Transfer to zero address.");
        }

        self.token_approvals.remove(&token_id);

        let from_balance = *self.owned_tokens_count.get(from).unwrap_or(&U256::zero());
        let to_balance = *self.owned_tokens_count.get(to).unwrap_or(&U256::zero());

        self.owned_tokens_count
            .insert(*from, from_balance.saturating_sub(U256::one()));
        self.owned_tokens_count
            .insert(*to, to_balance.saturating_add(U256::one()));

        self.token_owner.insert(token_id, *to);
>>>>>>> Add non-fungible token program

        let transfer_token = Transfer {
            from: H256::from_slice(from.as_ref()),
            to: H256::from_slice(to.as_ref()),
            token_id,
        };

        msg::reply(
<<<<<<< HEAD
            NftEvent::Transfer(transfer_token),
=======
            Event::Transfer(transfer_token),
>>>>>>> Add non-fungible token program
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

<<<<<<< HEAD
    fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256) {
        if spender == &ZERO_ID {
            panic!("NonFungibleToken: Approval to zero address.");
        }
        if spender == owner {
            panic!("NonFungibleToken: Approval to current owner");
        }
        if !self.is_token_owner(token_id, owner) {
=======
    fn transfer_from(&mut self, from: &ActorId, to: &ActorId, token_id: U256) {
        if !self.exists(token_id) {
            panic!("NonFungibleToken: token does not exist");
        }

        let source = msg::source();

        if !self.is_authorized_source(token_id, &source) {
            panic!("NonFungibleToken: is not an authorized source");
        }

        self.transfer(from, to, token_id);
        self.token_approvals.remove(&token_id);
    }

    fn approve(&mut self, spender: &ActorId, token_id: U256) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        let owner = msg::source();

        if spender == &zero {
            panic!("NonFungibleToken: Approval to zero address.");
        }

        if spender == &owner {
            panic!("NonFungibleToken: Approval to current owner");
        }

        if !self.is_token_owner(token_id, &owner) {
>>>>>>> Add non-fungible token program
            panic!("NonFungibleToken: is not owner");
        }

        self.token_approvals.insert(token_id, *spender);

        let approve_token = Approve {
            owner: H256::from_slice(owner.as_ref()),
            spender: H256::from_slice(spender.as_ref()),
            token_id,
        };
        msg::reply(
<<<<<<< HEAD
            NftEvent::Approval(approve_token),
=======
            Event::Approval(approve_token),
>>>>>>> Add non-fungible token program
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

<<<<<<< HEAD
    fn approve_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool) {
        if operator == &ZERO_ID {
=======
    fn approve_for_all(&mut self, operator: &ActorId, approved: bool) {
        let zero = ActorId::new(H256::zero().to_fixed_bytes());
        let owner = msg::source();

        if operator == &zero {
>>>>>>> Add non-fungible token program
            panic!("NonFungibleToken: Approval for a zero address");
        }

        self.operator_approval
<<<<<<< HEAD
            .entry(*owner)
=======
            .entry(owner)
>>>>>>> Add non-fungible token program
            .or_default()
            .insert(*operator, approved);

        let approve_operator = ApproveForAll {
            owner: H256::from_slice(owner.as_ref()),
            operator: H256::from_slice(operator.as_ref()),
            approved,
        };

        msg::reply(
<<<<<<< HEAD
            NftEvent::ApprovalForAll(approve_operator),
=======
            Event::ApprovalForAll(approve_operator),
>>>>>>> Add non-fungible token program
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
<<<<<<< HEAD
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
=======

    fn is_approved_for_all(&mut self, owner: &ActorId, operator: &ActorId) {
        let approved = self
            .operator_approval
            .get(owner)
            .and_then(|o| o.get(operator))
            .unwrap_or(&false);

        msg::reply(
            Event::IsApproved(*approved),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn get_approved(&mut self, token_id: U256) {
        if !self.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }

        let zero = ActorId::new(H256::zero().to_fixed_bytes());

        let approved_address = self.token_approvals.get(&token_id).unwrap_or(&zero);

        msg::reply(
            Event::ApprovedAddress(H256::from_slice(approved_address.as_ref())),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn owner_of(&mut self, token_id: U256) {
        if !self.token_owner.contains_key(&token_id) {
            panic!("NonFungibleToken: token doesn't exist");
        }

        let owner = self.token_owner.get(&token_id).unwrap();

        msg::reply(
            Event::Owner(H256::from_slice(owner.as_ref())),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn balance_of(&mut self, account_id: ActorId) {
        if account_id == ActorId::new(H256::zero().to_fixed_bytes()) {
            panic!("NonFungibleToken: requesting balance of zero address");
        }

        let zero = U256::zero();

        let balance = self.owned_tokens_count.get(&account_id).unwrap_or(&zero);

        msg::reply(
            Event::Balance(*balance),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

#[derive(Debug, Decode, TypeInfo)]
struct InitConfig {
    name: String,
    symbol: String,
    base_uri: String,
}

#[derive(Debug, Decode, TypeInfo)]
struct MintInput {
    account: H256,
}

#[derive(Debug, Decode, TypeInfo)]
struct BurnInput {
    account: H256,
    token_id: U256,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct ApproveInput {
    spender: H256,
    token_id: U256,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct Approve {
>>>>>>> Add non-fungible token program
    owner: H256,
    spender: H256,
    token_id: U256,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
<<<<<<< HEAD
pub struct ApproveForAll {
=======
struct ApproveForAllInput {
    operator: H256,
    approved: bool,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct ApproveForAll {
>>>>>>> Add non-fungible token program
    owner: H256,
    operator: H256,
    approved: bool,
}

<<<<<<< HEAD
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
=======
#[derive(Debug, Encode, Decode, TypeInfo)]
struct IsApprovedForAllInput {
    owner: H256,
    operator: H256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
struct TransferInput {
    from: H256,
    to: H256,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
struct Transfer {
    from: H256,
    to: H256,
    token_id: U256,
}

#[derive(Debug, Decode, TypeInfo)]
enum Action {
    Mint(MintInput),
    Burn(BurnInput),
    TokenURI(U256),
    TransferFrom(TransferInput),
    Approval(ApproveInput),
    SetApprovalForAll(ApproveForAllInput),
    GetApproved(U256),
    IsApprovedForAll(IsApprovedForAllInput),
    OwnerOf(U256),
    BalanceOf(H256),
}

#[derive(Debug, Encode, TypeInfo)]
enum Event {
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    Owner(H256),
    Balance(U256),
    TokenURI(String),
    IsApproved(bool),
    ApprovedAddress(H256),
}

gstd::metadata! {
    title: "NonFungibleToken",
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
            let to = ActorId::new(mint_input.account.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.mint(&to);
        }
        Action::Burn(burn_input) => {
            let from = ActorId::new(burn_input.account.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.burn(&from, burn_input.token_id);
        }
        Action::TokenURI(token_id) => {
            NON_FUNGIBLE_TOKEN.token_uri(token_id);
        }
        Action::Approval(approve) => {
            let spender = ActorId::new(approve.spender.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.approve(&spender, approve.token_id);
        }
        Action::SetApprovalForAll(approve) => {
            let operator = ActorId::new(approve.operator.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.approve_for_all(&operator, approve.approved);
        }
        Action::IsApprovedForAll(is_approved) => {
            let owner = ActorId::new(is_approved.owner.to_fixed_bytes());
            let operator = ActorId::new(is_approved.operator.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.is_approved_for_all(&owner, &operator);
        }
        Action::GetApproved(token_id) => {
            NON_FUNGIBLE_TOKEN.get_approved(token_id);
        }
        Action::TransferFrom(transfer) => {
            let from = ActorId::new(transfer.from.to_fixed_bytes());
            let to = ActorId::new(transfer.to.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.transfer_from(&from, &to, transfer.token_id);
        }
        Action::OwnerOf(token_id) => {
            NON_FUNGIBLE_TOKEN.owner_of(token_id);
        }
        Action::BalanceOf(account) => {
            let account_id = ActorId::new(account.to_fixed_bytes());
            NON_FUNGIBLE_TOKEN.balance_of(account_id);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NON_FUNGIBLE_TOKEN {:?}", config);
    NON_FUNGIBLE_TOKEN.set_name(config.name);
    NON_FUNGIBLE_TOKEN.set_symbol(config.symbol);
    NON_FUNGIBLE_TOKEN.set_base_uri(config.base_uri);
    debug!(
        "NON_FUNGIBLE_TOKEN {} SYMBOL {} BASE_URI {} created",
        NON_FUNGIBLE_TOKEN.name(),
        NON_FUNGIBLE_TOKEN.symbol(),
        NON_FUNGIBLE_TOKEN.base_uri()
    );
>>>>>>> Add non-fungible token program
}
