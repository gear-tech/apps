#![no_std]

use codec::{Decode, Encode};
use gear_contract_libraries::non_fungible_token::{royalties::*, token::*};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

pub type LayerId = u128;
pub type LayerItemId = u128;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum OnChainNFTAction {
    Mint {
        token_metadata: TokenMetadata,
        description: BTreeMap<LayerId, LayerItemId>,
    },
    Burn {
        token_id: TokenId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    Approve {
        to: ActorId,
        token_id: TokenId,
    },
    TransferPayout {
        to: ActorId,
        token_id: TokenId,
        amount: u128,
    },
    TokenURI {
        token_id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct TokenURI {
    pub metadata: TokenMetadata,
    pub content: Vec<String>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitOnChainNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub base_image: String,
    pub layers: BTreeMap<LayerId, BTreeMap<LayerItemId, String>>,
    pub royalties: Option<Royalties>,
}
