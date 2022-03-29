#![no_std]

extern crate proc_macro;
extern crate alloc;
#[allow(unused_imports)]
use derive_trait::{declare_derive_storage_trait};

// NFT
declare_derive_storage_trait!(derive_nft_storage, NFTStorage, NFTStorageField);

// Owner access 
declare_derive_storage_trait!(derive_owner_storage, OwnableStorage, OwnableStorageField);

