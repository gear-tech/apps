extern crate proc_macro;
extern crate syn;
extern crate quote;

#[proc_macro_derive(NFT, attributes(Action))]
pub fn derive_action(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive = syn::parse_macro_input!(_item as syn::DeriveInput);
    const TRAIT_NAME: &'static str = stringify!(NFTStorage);
    const FIELD_SETTER: &'static str = stringify!(NFTStorageField);

    let struct_ident = derive.ident;

    let field_ident;
    let field_ty;
    if let syn::Data::Struct(data) = &derive.data {
        if let syn::Fields::Named(named_fields) = &data.fields {
            let field = named_fields
                .named
                .iter()
                .find(|f| f.attrs.iter().find(|a| a.path.is_ident(FIELD_SETTER)).is_some());

            if let Some(field) = field {
                field_ident = field.ident.clone();
                field_ty = field.ty.clone();
            } else {
                let err_message = format!("Struct doesn't specify {} for trait {}", FIELD_SETTER, TRAIT_NAME);
                return quote::quote! {
                    compile_error!(#err_message);
                }
                .into()
            }
        } else {
            let err_message = format!("{} only supports named fields in struct", FIELD_SETTER);
            return quote::quote! {
                compile_error!(#err_message);
            }
            .into()
        }
    } else {
        let err_message = format!("{} only supports struct", FIELD_SETTER);
        return quote::quote! {
            compile_error!(#err_message);
        }
        .into()
    }

    let code = quote::quote! {
        impl $trait_name for #struct_ident {
            fn get(&self) -> & #field_ty {
                &self.#field_ident
            }

            fn get_mut(&mut self) -> &mut #field_ty {
                &mut self.#field_ident
            }
        }
    };
    code.into()
}