use proc_macro::{TokenStream};
use proc_macro2::{
    TokenStream as TokenStream2,
    TokenTree,
    Ident,
    Span,
};
use quote::{
    quote,
    quote_spanned,
    ToTokens,
};
use syn::{
    parse_macro_input,
    spanned::Spanned,
    ImplItemMethod,
};
extern crate alloc;
use alloc::{string::ToString, format};

const MODIFIER: &'static str = "modifier";
pub type ModifierMethod = syn::Path;

pub(crate) fn generate(_attrs: TokenStream, _input: TokenStream) -> TokenStream {
    let modifier = parse_macro_input!(_attrs as ModifierMethod);
    let mut impl_item =
        syn::parse2::<ImplItemMethod>(_input.into()).expect("Can't parse input of `modifiers` macro like a method.");

    let receiver;
    if let syn::FnArg::Receiver(rec) = impl_item.sig.inputs.first().unwrap() {
        receiver = rec;
    } else {
        return (quote_spanned! {
            impl_item.sig.inputs.first().unwrap().span() =>
                compile_error!("First argument in method must be `self`.");
        })
        .into()
    }
    let mut block = impl_item.block.clone();
    block = replace_self(block);
    let (final_block, body_ident) = put_into_closure(receiver, block);

    let stmts = final_block.stmts;
    block = syn::parse2::<syn::Block>(quote! {
            {
                #(#stmts)*
                #modifier(self, #body_ident)
            }
        })
        .unwrap();
    impl_item.block = block;

    let code = quote! {
        #impl_item
    };

    code.into()
}

fn replace_self(block: syn::Block) -> syn::Block {
    syn::parse2::<syn::Block>(recursive_replace_self(block.to_token_stream())).unwrap()
}

fn recursive_replace_self(token_stream: TokenStream2) -> TokenStream2 {
    token_stream
        .into_iter()
        .map(|token| {
            match &token {
                TokenTree::Ident(ident) => {
                    if ident.to_string() == "self" {
                        TokenTree::Ident(syn::Ident::new(MODIFIER, ident.span()))
                    } else {
                        token
                    }
                }
                TokenTree::Group(group) => {
                    let mut new_group =
                        proc_macro2::Group::new(group.delimiter(), recursive_replace_self(group.stream()));
                    new_group.set_span(group.span());
                    TokenTree::Group(new_group)
                }
                _ => token,
            }
        })
        .collect()
}

fn put_into_closure(receiver: &syn::Receiver, block: syn::Block) -> (syn::Block, syn::Ident) {
    let body_ident = Ident::new(&format!("body_function"), Span::call_site());
    let instance_ident = syn::Ident::new(MODIFIER, receiver.span());

    let reference = match receiver.mutability.is_some() {
        true => quote! { &mut },
        false => quote! { & },
    };

    // Put the body of original function to local lambda function
    let final_block = syn::parse2::<syn::Block>(quote! {
        {
            let mut #body_ident = |#instance_ident: #reference Self| #block;
        }
    })
    .unwrap();

    (final_block, body_ident)
}