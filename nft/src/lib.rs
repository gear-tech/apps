#![no_std]

use gear_contract_libraries::non_fungible_token::{nft_core::*, state::*, token::*};
use gstd::{msg, prelude::*, ActorId};
use nft_io::*;
use primitive_types::U256;
use derive_traits::{NFTStateKeeper, NFTCore, NFTMetaState};

#[derive(Debug, Default, NFTStateKeeper, NFTCore, NFTMetaState)]
pub struct NFT {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
}

static mut CONTRACT: Option<NFT> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitNFT = msg::load().expect("Unable to decode InitConfig");
    let mut nft = NFT::default();
    nft.token.name = config.name;
    nft.token.symbol = config.symbol;
    nft.token.base_uri = config.base_uri;
    nft.owner = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Vec<u8> = msg::load().expect("Could not load msg");
    let nft = CONTRACT.get_or_insert(NFT::default());
    MyNFTCore::proc(nft, action);
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: Vec<u8> = msg::load().expect("failed to decode input argument");
    let nft = CONTRACT.get_or_insert(NFT::default());
    let encoded = NFTMetaState::proc_state(nft, query).expect("error");
    gstd::util::to_leak_ptr(encoded)
}

pub trait MyNFTCore: NFTCore {
    fn mint(&mut self, token_metadata: TokenMetadata);

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() < 2 {
            return None;
        }
        if bytes[0] == 0 {
            let mut bytes = bytes;
            bytes.remove(0);
            return <Self as NFTCore>::proc(self, bytes);
        }
        let action = MyNFTAction::decode(&mut &bytes[..]).ok()?;
        match action {
            MyNFTAction::Mint { token_metadata } => <Self as MyNFTCore>::mint(self, token_metadata),
            MyNFTAction::Base(_) => unreachable!(),
        }
        Some(())
    }
}

impl MyNFTCore for NFT {
    fn mint(&mut self, token_metadata: TokenMetadata) {
        NFTCore::mint(self, &msg::source(), self.token_id, Some(token_metadata));
        self.token_id = self.token_id.saturating_add(U256::one());
    }
}
