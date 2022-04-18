#![no_std]

extern crate proc_macro;
extern crate alloc;
#[allow(unused_imports)]
use macros::*;
//use proc_macro::TokenStream;
//use proc_macro2::{Span, Ident,TokenStream as TokenStream2};
//use quote::{quote,  quote_spanned};
//use syn::spanned::Spanned;
//use convert_case::{Case, Casing};
// /use alloc::format;
//use crate::alloc::string::ToString;
//use syn::{Data, parse_macro_input, DeriveInput};
// NFT
declare_derive_storage_trait!(derive_nft_state, NFTStateKeeper, NFTStateField);
declare_impl_trait!(derive_nft_core, NFTCore);
declare_impl_trait!(derive_nft_metastate, NFTMetaState);




