#![no_std]
use multitoken_io::*;
use gear_contract_libraries::multitoken::{io::*, mtk_core::*, state::*};
use gstd::{msg, prelude::*, ActorId};

#[derive(Debug, Default)]
pub struct SimpleMTK {
    pub tokens: MTKState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub supply: BTreeMap<TokenId, u128>,
}

impl StateKeeper for SimpleMTK {
    fn get(&self) -> &MTKState {
        &self.tokens
    }
    fn get_mut(&mut self) -> &mut MTKState {
        &mut self.tokens
    }
}

impl BalanceTrait for SimpleMTK {}
impl MTKTokenState for SimpleMTK {}
impl MTKTokenAssert for SimpleMTK {}
impl MTKCore for SimpleMTK {}

pub trait SimpleMTKCore: MTKCore {
    fn mint(&mut self, amount: u128, token_metadata: Option<TokenMetadata>);

    fn burn(&mut self, id: TokenId, amount: u128);

    fn supply(&mut self, id: TokenId) -> u128;

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }

        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as MTKCore>::proc(self, bytes);
        }
        let action = MyMTKAction::decode(&mut &bytes[..]).ok()?;
        match action {
            MyMTKAction::Mint {
                amount,
                token_metadata,
            } => <Self as SimpleMTKCore>::mint(self, amount, token_metadata),
            MyMTKAction::Burn { id, amount } => <Self as SimpleMTKCore>::burn(self, id, amount),
            MyMTKAction::Supply { id } => {
                msg::reply(
                    MyMTKEvent::Supply {
                        amount: <Self as SimpleMTKCore>::supply(self, id),
                    },
                    0,
                )
                .unwrap();
            }
            MyMTKAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}

static mut CONTRACT: Option<SimpleMTK> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitMTK = msg::load().expect("Unable to decode InitConfig");
    let mut multi_token = SimpleMTK::default();
    multi_token.tokens.name = config.name;
    multi_token.tokens.symbol = config.symbol;
    multi_token.tokens.base_uri = config.base_uri;
    multi_token.owner = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Vec<u8> = msg::load().expect("Could not load msg");
    let multi_token = CONTRACT.get_or_insert(SimpleMTK::default());
    SimpleMTKCore::proc(multi_token, action);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: Vec<u8> = msg::load().expect("failed to decode input argument");
    let multi_token = CONTRACT.get_or_insert(SimpleMTK::default());
    let encoded = MTKTokenState::proc_state(multi_token, query).expect("error");
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
