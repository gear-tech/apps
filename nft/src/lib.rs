#![no_std]
#![feature(const_btree_new)]

use codec::Encode;
use gstd::{debug, msg, prelude::*, ActorId};
use primitive_types::U256;
use gear_contract_libraries::non_fungible_token::traits::NonFungibleTokenBase;
use gear_contract_libraries::non_fungible_token::nft_core::*;
use gear_contract_libraries::non_fungible_token::io::*;
use gear_contract_libraries::access::owner_access::*;

#[derive(Debug, Default, NFTStorage, OwnableStorage)]
pub struct NFT {
    #[NFTStorageField]
    pub token: NFTData,
    pub token_id: U256,
    #[OwnableStorageField]
    pub owner: OwnableData,
}

static mut CONTRACT: Option<NFT> = None;

impl  NFT {   

    #[modifier(only_owner)]
    fn mint_token(&mut self) {
        self.mint(&msg::source(), self.token_id);
        self.token_id = self.token_id.saturating_add(U256::one());
    }
}


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
    let action: NFTAction = msg::load().expect("Could not load Action");
    let nft = CONTRACT.get_or_insert(NFT::default());
    match action {
        NFTAction::Mint => {
            nft.mint_token();
        }
        NFTAction::Burn(token_id) => {
            nft.burn(token_id);
        }
        NFTAction::Transfer { to, token_id } => {
            nft.transfer(&to, token_id);
        }
        NFTAction::Approve { to, token_id } => {
            nft.approve(&to, token_id);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    let mut nft = NFT::default();
    nft.token.name = config.name;
    nft.token.symbol = config.symbol;
    nft.token.base_uri = config.base_uri;
    nft.owner = OwnableData{ owner: msg::source()};
 }

 #[derive(Debug, Encode, Decode, TypeInfo)]
 pub enum NFTAction {
     Mint,
     Burn(U256),
     Transfer { to: ActorId, token_id: U256 },
     Approve { to: ActorId, token_id: U256 },
 }