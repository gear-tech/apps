#![no_std]
extern crate proc_macro;
use proc_macro::TokenStream;

mod modifier;

#[proc_macro_attribute]
pub fn modifier(_attrs: TokenStream, method: TokenStream) -> TokenStream {
    modifier::generate(_attrs, method)
}
