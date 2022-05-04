#![no_std]
use gear_contract_libraries::multitoken::{io::*, mtk_core::*, state::*};
use gstd::{debug, msg, prelude::*, ActorId};
use multitoken_io::*;
use derive_traits::{BalanceTrait, MTKTokenState, MTKTokenAssert, MTKCore, StateKeeper};

#[derive(Debug, Default, BalanceTrait, MTKTokenState, MTKTokenAssert, MTKCore, StateKeeper)]
pub struct SimpleMTK {
    #[MTKStateKeeper]
    pub tokens: MTKState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub supply: BTreeMap<TokenId, u128>,
}

pub trait SimpleMTKCore: MTKCore {
    fn mint(&mut self, amount: u128, token_metadata: Option<TokenMetadata>);

    fn burn(&mut self, id: TokenId, amount: u128);

    fn supply(&mut self, id: TokenId) -> u128;
}

static mut CONTRACT: Option<SimpleMTK> = None;

gstd::metadata! {
    title: "MTK",
    init:
        input: InitMTK,
    handle:
        input: MyMTKAction,
        output: Vec<u8>,
    state:
        input: MTKQuery,
        output: MTKQueryReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitMTK = msg::load().expect("Unable to decode InitConfig");
    let mut multi_token = SimpleMTK::default();
    multi_token.tokens.name = config.name;
    multi_token.tokens.symbol = config.symbol;
    multi_token.tokens.base_uri = config.base_uri;
    multi_token.owner = msg::source();
    CONTRACT = Some(multi_token);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: MyMTKAction = msg::load().expect("Could not load msg");
    let multi_token = CONTRACT.get_or_insert(SimpleMTK::default());
    match action {
        MyMTKAction::Mint {
            amount,
            token_metadata,
        } => SimpleMTKCore::mint(multi_token, amount, token_metadata),
        MyMTKAction::Burn { id, amount } => SimpleMTKCore::burn(multi_token, id, amount),
      //  MyMTKAction::Supply { id } => SimpleMTKCore::supply(multi_token, id),
        MyMTKAction::BalanceOf { account, id } => MTKCore::balance_of(multi_token, &account, &id),
        MyMTKAction::BalanceOfBatch { accounts, ids } => {
            MTKCore::balance_of_batch(multi_token, &accounts, &ids)
        }
        MyMTKAction::MintBatch {
            amounts,
            ids,
            tokens_metadata,
        } => MTKCore::mint_batch(multi_token, &msg::source(), &amounts, &ids, tokens_metadata),
        MyMTKAction::TransferFrom {
            from,
            to,
            id,
            amount,
        } => MTKCore::transfer_from(multi_token, &from, &to, &id, amount),
        MyMTKAction::BatchTransferFrom {
            from,
            to,
            ids,
            amounts,
        } => MTKCore::batch_transfer_from(multi_token, &from, &to, &ids, &amounts),
        MyMTKAction::BurnBatch { ids, amounts } => MTKCore::burn_batch(multi_token, &ids, &amounts),
        MyMTKAction::Approve { account } => MTKCore::approve(multi_token, &account),
        MyMTKAction::RevokeApproval { account } => MTKCore::revoke_approval(multi_token, &account),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: MTKQuery = msg::load().expect("failed to decode input argument");
    // let query: Vec<u8> = msg::load().expect("failed to decode input argument");
    let query_bytes = query.encode();
    let multi_token = CONTRACT.get_or_insert(SimpleMTK::default());
    let encoded = MTKTokenState::proc_state(multi_token, query_bytes).expect("error");
    gstd::util::to_leak_ptr(encoded)
}

impl SimpleMTKCore for SimpleMTK {
    fn mint(&mut self, amount: u128, token_metadata: Option<TokenMetadata>) {
        MTKCore::mint(
            self,
            &msg::source(),
            &(self.token_id.clone()),
            amount,
            token_metadata,
        );
        self.supply.insert(self.token_id, amount);
        self.token_id = self.token_id.saturating_add(1);
    }

    fn burn(&mut self, id: TokenId, amount: u128) {
        MTKCore::burn(self, &id, amount);
        let sup = self.supply(id);
        let mut _balance = self
            .supply
            .insert(self.token_id, sup.saturating_sub(amount));
    }

    fn supply(&mut self, id: TokenId) -> u128 {
        *self.supply.entry(id).or_default()
    }
}
