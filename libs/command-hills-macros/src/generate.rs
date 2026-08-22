use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Meta, Result, Token};

use crate::model::{Declaration, Field};

pub(crate) fn generate(declaration: Declaration) -> Result<TokenStream> {
    let visibility = &declaration.visibility;
    let ident = &declaration.ident;
    let generics = &declaration.generics;
    let target = &declaration.target;
    let context = &declaration.context;
    let struct_attributes = &declaration.clap_attributes;
    let mut fields = declaration.fields.iter().collect::<Vec<_>>();
    fields.sort_by_key(|field| field.position);
    let generated_fields = fields.iter().map(|field| generate_field(field));
    let resolved_fields = fields
        .iter()
        .filter(|field| field.markers.is_empty())
        .map(|field| resolve_field(field))
        .collect::<Result<Vec<_>>>()?;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[derive(clap::Parser)]
        #(#struct_attributes)*
        #visibility struct #ident #generics {
            #(#generated_fields,)*
        }

        impl #impl_generics #ident #type_generics #where_clause {
            pub fn resolve(self, ctx: &#context) -> #target {
                let _ = ctx;
                #target {
                    #(#resolved_fields,)*
                }
            }
        }
    })
}

fn generate_field(field: &Field) -> TokenStream {
    let visibility = &field.visibility;
    let ident = &field.ident;
    let ty = field
        .markers
        .iter()
        .find(|marker| marker.ident == "Only")
        .map_or(&field.ty, |marker| &marker.inner_type);
    let attributes = &field.clap_attributes;

    quote! {
        #(#attributes)*
        #visibility #ident: #ty
    }
}

fn resolve_field(field: &Field) -> Result<TokenStream> {
    let ident = &field.ident;

    if is_flattened(field)? {
        Ok(quote!(#ident: ::command_hills::Resolve::resolve(self.#ident)))
    } else {
        Ok(quote!(#ident: self.#ident))
    }
}

fn is_flattened(field: &Field) -> Result<bool> {
    for attribute in &field.clap_attributes {
        if !attribute.path().is_ident("command") {
            continue;
        }

        let entries = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        if entries
            .iter()
            .any(|entry| matches!(entry, Meta::Path(path) if path.is_ident("flatten")))
        {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use quote::{ToTokens, quote};

    use super::*;
    use crate::parser::parse_declaration;

    #[test]
    fn generates_parser_and_resolve() {
        let declaration = parse_declaration(
            quote!(target = Action::Start, context = Docker),
            quote! {
                pub struct Start {
                    #[arg(long)]
                    pub base: Option<Base>,
                    #[command(flatten)]
                    pub prompt: PromptArgs,
                }
            },
        )
        .expect("объявление должно разбираться");
        let generated = generate(declaration).expect("объявление должно порождать код");
        let rendered = generated.to_token_stream().to_string();

        assert!(rendered.contains("derive (clap :: Parser)"));
        assert!(rendered.contains("base : self . base"));
        assert!(
            rendered.contains("prompt : :: command_hills :: Resolve :: resolve (self . prompt)")
        );
        assert!(rendered.contains("ctx : & Docker"));
    }

    #[test]
    fn removes_hill_attributes() {
        let declaration = parse_declaration(
            quote!(target = Action::Start, context = Docker),
            quote! {
                struct Start {
                    #[arg(long)]
                    #[hill(keep = "Base")]
                    base: Option<Base>,
                }
            },
        )
        .expect("объявление должно разбираться");
        let generated = generate(declaration).expect("объявление должно порождать код");
        let rendered = generated.to_string();

        assert!(rendered.contains("arg (long)"));
        assert!(!rendered.contains("hill"));
    }

    #[test]
    fn converts_only_marker_and_skips_target_field() {
        let declaration = parse_declaration(
            quote!(target = Action::Start, context = Docker),
            quote! {
                struct Start {
                    #[arg(long)]
                    connect: Only<String>,
                }
            },
        )
        .expect("объявление должно разбираться");
        let generated = generate(declaration).expect("объявление должно порождать код");
        let rendered = generated.to_string();

        assert!(rendered.contains("connect : String"));
        assert!(!rendered.contains("connect : self . connect"));
    }

    #[test]
    fn malformed_command_attribute_is_reported() {
        let declaration = parse_declaration(
            quote!(target = Action::Start, context = Docker),
            quote! {
                struct Start {
                    #[command(flatten =)]
                    prompt: PromptArgs,
                }
            },
        )
        .expect("объявление должно разбираться");
        let error = generate(declaration).expect_err("некорректный command должен давать ошибку");

        assert!(error.to_string().contains("expected"));
    }
}
