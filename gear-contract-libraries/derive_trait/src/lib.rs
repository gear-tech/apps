#![no_std]

extern crate proc_macro;
extern crate alloc;
#[allow(unused_imports)]
use utils::declare_derive_storage_trait;

use proc_macro::TokenStream;
use proc_macro2::{Span, Ident,TokenStream as TokenStream2};
use quote::{quote,  quote_spanned};
use syn::spanned::Spanned;
use convert_case::{Case, Casing};
use alloc::format;
use crate::alloc::string::ToString;


use syn::{Data, parse_macro_input, DeriveInput};
// NFT
declare_derive_storage_trait!(derive_nft_storage, NFTStorage, NFTStorageField);

#[proc_macro_derive(ActionParser)]
pub fn derive_action_trait(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    // get enum name
    let ref name = input.ident;
    let ref data = input.data;

    let mut variant_checker_functions;

    match data {
        Data::Enum(data_enum) => {
            variant_checker_functions = TokenStream2::new();

            // Iterate over enum variants
            for variant in &data_enum.variants {

                // Variant's name
                let ref variant_name = variant.ident;

                let variant_func_name =
                    format!("action_{}", variant_name.to_string().to_case(Case::Snake));
                let action_variant_func_name = Ident::new(&variant_func_name, Span::call_site());
                variant_checker_functions.extend(quote_spanned! {variant.span()=>
                    fn #action_variant_func_name(&mut self, action: #name) {
                    }
                });
            }
        }
        _ => panic!("ActionParser is only implemented for enums"),
    };
    let gen = quote! {
        pub trait NFTActionParser {
            #variant_checker_functions
        }
        
    };
    gen.into()
}




