use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, ItemStruct, Error};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn generate_resolve(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let mut fields = quote! {};
    for field in &input.fields {
        let field_name = &field.ident;
        
        for attr in &field.attrs {
            if attr.path().is_ident("unknown") {
                return Error::new(attr.span(), "unknown attribute").to_compile_error().into();
            }
        }
        
        if field_name.as_ref().map(|i| i == "base").unwrap_or(false) {
            continue;
        }

        fields.extend(quote_spanned! { field.span() =>
            #field_name: self.#field_name,
        });
    }

    let target_init = quote_spanned! { name.span() =>
        Target {
            #fields
        }
    };

    let expanded = quote! {
        #input

        impl #name {
            pub fn resolve(self) -> Target {
                #target_init
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ResolveDerive, attributes(unknown))]
pub fn derive_resolve(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let mut fields = quote! {};
    for field in &input.fields {
        let field_name = &field.ident;
        
        for attr in &field.attrs {
            if attr.path().is_ident("unknown") {
                return Error::new(attr.span(), "unknown attribute").to_compile_error().into();
            }
        }
        
        if field_name.as_ref().map(|i| i == "base").unwrap_or(false) {
            continue;
        }

        fields.extend(quote_spanned! { field.span() =>
            #field_name: self.#field_name,
        });
    }

    let target_init = quote_spanned! { name.span() =>
        Target {
            #fields
        }
    };

    let expanded = quote! {
        impl #name {
            pub fn resolve(self) -> Target {
                #target_init
            }
        }
    };

    TokenStream::from(expanded)
}
