#![no_std]

use derive_traits::{NFTCore, NFTMetaState, NFTStateKeeper};
use gear_contract_libraries::non_fungible_token::{nft_core::*, state::*, token::*};
use gstd::{msg, prelude::*, ActorId};
use on_chain_nft_io::*;
use primitive_types::U256;


#[derive(Debug, Default, NFTStateKeeper, NFTCore, NFTMetaState)]
pub struct OnChainNFT {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub base_image: Vec<u8>,
    pub layers: BTreeMap<LayerId, BTreeMap<LayerItemId, Vec<u8>>>,
    pub nfts: BTreeMap<TokenId, BTreeMap<LayerId, LayerItemId>>,
}

static mut CONTRACT: Option<OnChainNFT> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitOnChainNFT = msg::load().expect("Unable to decode InitNFT");
    let mut _layers: BTreeMap<LayerId, BTreeMap<LayerItemId, Vec<u8>>> = BTreeMap::new();
    for (layer_id, layer) in config.layers.iter() {
        let mut layer_map: BTreeMap<LayerItemId, Vec<u8>> = BTreeMap::new();
        for (layer_item_id, layer_item) in layer.clone() {
            layer_map.insert(layer_item_id, layer_item.into_bytes());
        }
        _layers.insert(*layer_id, layer_map);
    }
    let nft = OnChainNFT {
        token: NFTState {
            name: config.name,
            symbol: config.symbol,
            base_uri: config.base_uri,
            ..Default::default()
        },
        owner: msg::source(),
        base_image: config.base_image.into_bytes(),
        layers: _layers,
        ..Default::default()
    };
    CONTRACT = Some(nft);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: OnChainNFTAction = msg::load().expect("Could not load NFTAction");
    let nft = CONTRACT.get_or_insert(Default::default());
    match action {
        OnChainNFTAction::Mint { description, token_metadata } => OnChainNFTCore::mint(nft, description, token_metadata),
        OnChainNFTAction::Burn { token_id } => OnChainNFTCore::burn(nft, token_id),
        OnChainNFTAction::TokenURI { token_id } => OnChainNFTCore::token_uri(nft, token_id),
        OnChainNFTAction::Transfer { to, token_id } => NFTCore::transfer(nft, &to, token_id),
        OnChainNFTAction::TransferPayout {
            to,
            token_id,
            amount,
        } => NFTCore::transfer_payout(nft, &to, token_id, amount),
        OnChainNFTAction::Approve { to, token_id } => NFTCore::approve(nft, &to, token_id),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: NFTQuery = msg::load().expect("failed to decode input argument");
    let nft = CONTRACT.get_or_insert(OnChainNFT::default());
    let encoded =
        NFTMetaState::proc_state(nft, query).expect("Error in reading NFT contract state");
    gstd::util::to_leak_ptr(encoded)
}

pub trait OnChainNFTCore: NFTCore {
    fn mint(&mut self, description: BTreeMap<LayerId, LayerItemId>, metadata: TokenMetadata);
    fn burn(&mut self, token_id: TokenId);
    fn token_uri(&mut self, token_id: TokenId);
}

impl OnChainNFTCore for OnChainNFT {
    fn mint(&mut self, description: BTreeMap<LayerId, LayerItemId>, metadata: TokenMetadata) {
        NFTCore::mint(self, &msg::source(), self.token_id, Some(metadata));
        self.nfts.insert(self.token_id, description);
        self.token_id = self.token_id.saturating_add(U256::one());
    }

    fn burn(&mut self, token_id: TokenId) {
        NFTCore::burn(self, token_id);
        self.nfts.remove(&token_id);
    }

    fn token_uri(&mut self, token_id: TokenId) {
        let mut metadata = TokenMetadata::default();
        if let Some(Some(mtd)) = self.token.token_metadata_by_id.get(&token_id) {
            metadata = mtd.clone();
        }
        // construct media
        let mut content: Vec<String> = Vec::new();
        // check if exists
        let nft = self.nfts.get(&token_id).expect("Not suc nft");
        for (layer_id, layer_item_id) in nft {
            let layer_content = self.layers.get(layer_id).expect("No such layer").get(layer_item_id).expect("No such layer item");
            let cc = String::from_utf8((*layer_content).clone()).expect("Found invalid UTF-8");
            content.push(cc);
        }
        msg::reply(
            TokenURI {
                metadata,
                content,
            }
            .encode(),
            0,
        )
        .unwrap();
    }
}

gstd::metadata! {
    title: "OnChainNFT",
    init:
        input: InitOnChainNFT,
    handle:
        input: OnChainNFTAction,
        output: Vec<u8>,
    state:
        input: NFTQuery,
        output: NFTQueryReply,
}
