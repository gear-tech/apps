#![no_std]

use macros::*;
// NFT
declare_derive_storage_trait!(derive_nft_state, NFTStateKeeper, NFTStateField);
declare_impl_trait!(derive_nft_core, NFTCore);
declare_impl_trait!(derive_nft_metastate, NFTMetaState);
