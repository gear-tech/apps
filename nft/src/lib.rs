#![no_std]

use codec::Encode;
use gear_contract_libraries::non_fungible_token::{io::*, nft_core::*};
use gstd::{msg, prelude::*, ActorId};
use primitive_types::U256;

#[derive(Debug, Default)]
pub struct NFT {
    pub token: NFTState,
    pub token_id: U256,
    pub owner: ActorId,
}

impl StateKeeper for NFT {
    fn get(&self) -> &NFTState {
        &self.token
    }
    fn get_mut(&mut self) -> &mut NFTState {
        &mut self.token
    }
}

static mut CONTRACT: Option<NFT> = None;

gstd::metadata! {
    title: "NFT",
        init:
            input: InitConfig,
        handle:
            input: NFTAction,
            output: NFTEvent,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Vec<u8> = msg::load().expect("Could not load msg");
    let nft = CONTRACT.get_or_insert(NFT::default());
    MyNFTCore::proc(nft, action);
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    let mut nft = NFT::default();
    nft.token.name = config.name;
    nft.token.symbol = config.symbol;
    nft.token.base_uri = config.base_uri;
    nft.owner = msg::source();
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MyNFTAction {
    Mint,
    Base(NFTAction),
}

pub trait MyNFTCore: NFTCore {
    fn mint(&mut self);

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }
        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as MyNFTCore>::proc(self, bytes);
        }
        let action = MyNFTAction::decode(&mut &bytes[..]).ok()?;
        match action {
            MyNFTAction::Mint => <Self as MyNFTCore>::mint(self),
            MyNFTAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}
impl NonFungibleTokenAssert for NFT {}
impl NFTCore for NFT {}
impl MyNFTCore for NFT {
    fn mint(&mut self) {
        NFTCore::mint(self, &msg::source(), self.token_id);
        self.token_id = self.token_id.saturating_add(U256::one());
    }
}
