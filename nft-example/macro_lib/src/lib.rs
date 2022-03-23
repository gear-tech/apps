// necessary for the TokenStream::from_str() implementation
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemStruct;
