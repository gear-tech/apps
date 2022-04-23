#![no_std]
use erc1155_io::*;
use gear_contract_libraries::erc1155::{erc1155_core::*, state::*, io::*};
use gstd::{msg, prelude::*, ActorId};


#[derive(Debug, Default)]
pub struct SimpleERC1155 {
    pub tokens: ERC1155State,
    pub token_id: TokenId,
    pub owner: ActorId,
}

impl StateKeeper for SimpleERC1155 {
    fn get(&self) -> &ERC1155State {
        &self.tokens
    }
    fn get_mut(&mut self) -> &mut ERC1155State {
        &mut self.tokens
    }
}

impl BalanceTrait for SimpleERC1155 {}
impl ERC1155TokenState for SimpleERC1155 {}
impl ERC1155TokenAssert for SimpleERC1155 {}
impl ERC1155Core for SimpleERC1155 {}

pub trait SimpleERC1155Core: ERC1155Core {
    fn mint(
        &mut self,
        amount: u128,
        token_metadata: Option<TokenMetadata>,
    );

    fn burn(&mut self, id: TokenId, amount: u128);

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }

        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as ERC1155Core>::proc(self, bytes);
        }
        let action = ERC1155Action::decode(&mut &bytes[..]).ok()?;
        match action {
            ERC1155Action::Mint {
                amount,
                token_metadata,
            } => <Self as SimpleERC1155Core>::mint(self, amount, token_metadata),
            ERC1155Action::Burn { id, amount } => {
                <Self as SimpleERC1155Core>::burn(self, id, amount)
            }
            ERC1155Action::Base(_) => unreachable!(),
        }
        Some(())
    }
}


static mut CONTRACT: Option<SimpleERC1155> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitERC1155 = msg::load().expect("Unable to decode InitConfig");
    let mut multi_token = SimpleERC1155::default();
    multi_token.tokens.name = config.name;
    multi_token.tokens.symbol = config.symbol;
    multi_token.tokens.base_uri = config.base_uri;
    multi_token.owner = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Vec<u8> = msg::load().expect("Could not load msg");
    let multi_token = CONTRACT.get_or_insert(SimpleERC1155::default());
    SimpleERC1155Core::proc(multi_token, action);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: Vec<u8> = msg::load().expect("failed to decode input argument");
    let multi_token = CONTRACT.get_or_insert(SimpleERC1155::default());
    let encoded = ERC1155TokenState::proc_state(multi_token, query).expect("error");
    gstd::util::to_leak_ptr(encoded)
}


impl SimpleERC1155Core for SimpleERC1155 {
    fn mint(
        &mut self,
        amount: u128,
        token_metadata: Option<TokenMetadata>,
    ) {
        ERC1155Core::mint(self, &msg::source(), &(self.token_id.clone()), amount, token_metadata);
        self.token_id = self.token_id.saturating_add(1);
    }

    fn burn(&mut self, id: TokenId, amount: u128) {
        ERC1155Core::burn(self, &id, amount);
    }
}
