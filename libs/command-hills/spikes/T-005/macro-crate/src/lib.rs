use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Expr};

#[proc_macro_attribute]
pub fn generate_cli(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let mut fields = quote! {};

    for field in &mut input.fields {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let mut clap_args = quote! { long };

        let mut retain_attrs = vec![];
        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("hill") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("candidates") {
                        let value: Expr = meta.value()?.parse()?;
                        clap_args.extend(quote! {
                            , add = ::spike_lib::reexports::ArgValueCandidates::new(#value)
                        });
                    }
                    if meta.path.is_ident("completer") {
                        let value: Expr = meta.value()?.parse()?;
                        clap_args.extend(quote! {
                            , add = ::spike_lib::reexports::ArgValueCompleter::new(#value)
                        });
                    }
                    if meta.path.is_ident("value_parser") {
                        let value: Expr = meta.value()?.parse()?;
                        clap_args.extend(quote! {
                            , value_parser = #value
                        });
                    }
                    Ok(())
                }).unwrap();
            } else {
                retain_attrs.push(attr);
            }
        }
        field.attrs = retain_attrs;

        fields.extend(quote! {
            #[arg(#clap_args)]
            pub #field_name: #field_type,
        });
    }

    let expanded = quote! {
        #[derive(clap::Parser, Debug)]
        pub struct #name {
            #fields
        }
    };

    TokenStream::from(expanded)
}
