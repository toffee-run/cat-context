use heck::{ToKebabCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Fields, ItemEnum, Meta, Path, Result, Token, parenthesized};

pub(crate) fn expand(arguments: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let mut item = syn::parse2::<ItemEnum>(input)?;
    let options = syn::parse2::<Options>(arguments)?;
    let fallback = options.fallback.ok_or_else(|| {
        Error::new(
            item.ident.span(),
            format!(
                "перечисление `{}` должно указать `fallback` в макросе group",
                item.ident
            ),
        )
    })?;
    let enum_ident = item.ident.clone();
    let args_ident = format_ident!("{}Args", enum_ident);
    let visibility = item.vis.clone();
    let mut generated_fields = Vec::new();
    let mut required_branches = Vec::new();
    let mut optional_branches = Vec::new();

    for variant in &mut item.variants {
        let attributes = parse_attributes(&variant.attrs, &variant.ident)?;
        variant
            .attrs
            .retain(|attribute| !attribute.path().is_ident("hill"));
        let field_ident = format_ident!(
            "{}",
            variant.ident.to_string().to_snake_case(),
            span = variant.ident.span()
        );
        let flag_name = variant.ident.to_string().to_kebab_case();
        let flag_name = syn::LitStr::new(&flag_name, variant.ident.span());
        let default_long = if has_long(&attributes)? {
            None
        } else {
            Some(quote_spanned! {variant.ident.span()=> #[arg(long = #flag_name)]})
        };
        let variant_ident = &variant.ident;

        match &variant.fields {
            Fields::Unit => {
                generated_fields.push(quote_spanned! {variant.ident.span()=>
                    #default_long
                    #(#attributes)*
                    #visibility #field_ident: bool
                });
                required_branches.push(quote_spanned! {variant.ident.span()=>
                    if self.#field_ident {
                        return Ok(#enum_ident::#variant_ident);
                    }
                });
                optional_branches.push(quote_spanned! {variant.ident.span()=>
                    if self.#field_ident {
                        return Ok(Some(#enum_ident::#variant_ident));
                    }
                });
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = fields
                    .unnamed
                    .first()
                    .map(|field| &field.ty)
                    .ok_or_else(|| Error::new(variant.ident.span(), "ожидалось поле варианта"))?;
                generated_fields.push(quote_spanned! {variant.ident.span()=>
                    #default_long
                    #(#attributes)*
                    #visibility #field_ident: Option<#ty>
                });
                required_branches.push(quote_spanned! {variant.ident.span()=>
                    if let Some(value) = self.#field_ident {
                        return Ok(#enum_ident::#variant_ident(value));
                    }
                });
                optional_branches.push(quote_spanned! {variant.ident.span()=>
                    if let Some(value) = self.#field_ident {
                        return Ok(Some(#enum_ident::#variant_ident(value)));
                    }
                });
            }
            Fields::Unnamed(fields) => {
                return Err(Error::new_spanned(
                    fields,
                    format!(
                        "вариант `{variant_ident}` в group должен содержать ровно одно безымянное поле"
                    ),
                ));
            }
            Fields::Named(fields) => {
                return Err(Error::new_spanned(
                    fields,
                    format!(
                        "вариант `{variant_ident}` в group должен быть без полей или содержать одно безымянное поле"
                    ),
                ));
            }
        }
    }

    Ok(quote! {
        #item

        #[derive(clap::Args)]
        #[group(multiple = false)]
        #visibility struct #args_ident {
            #(#generated_fields,)*
        }

        impl ::command_hills::Resolve<#enum_ident> for #args_ident {
            fn resolve(self) -> ::command_hills::Result<#enum_ident> {
                #(#required_branches)*
                #fallback()
            }
        }

        impl ::command_hills::Resolve<Option<#enum_ident>> for #args_ident {
            fn resolve(self) -> ::command_hills::Result<Option<#enum_ident>> {
                #(#optional_branches)*
                Ok(None)
            }
        }
    })
}

fn parse_attributes(attributes: &[Attribute], variant: &syn::Ident) -> Result<Vec<Attribute>> {
    let mut clap_attributes = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("hill"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("arg") {
                return Err(meta.error(format!(
                    "неизвестная пометка варианта `{variant}`; допустима только `arg`"
                )));
            }
            let content;
            parenthesized!(content in meta.input);
            let tokens: TokenStream = content.parse()?;
            clap_attributes.extend(Attribute::parse_outer.parse2(quote_spanned! {
                meta.path.span()=> #[arg(#tokens)]
            })?);
            Ok(())
        })?;
    }
    Ok(clap_attributes)
}

fn has_long(attributes: &[Attribute]) -> Result<bool> {
    for attribute in attributes {
        let entries = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        if entries.iter().any(|entry| entry.path().is_ident("long")) {
            return Ok(true);
        }
    }
    Ok(false)
}

struct Options {
    fallback: Option<Path>,
}

impl syn::parse::Parse for Options {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        let entries = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut fallback = None;
        for entry in entries {
            let Meta::NameValue(value) = entry else {
                return Err(Error::new_spanned(
                    entry,
                    "ожидался параметр в форме `fallback = путь`",
                ));
            };
            if !value.path.is_ident("fallback") {
                return Err(Error::new_spanned(
                    value.path,
                    "неизвестный параметр group; допустим только `fallback`",
                ));
            }
            if fallback.is_some() {
                return Err(Error::new_spanned(
                    value.path,
                    "параметр `fallback` указан несколько раз",
                ));
            }
            let syn::Expr::Path(path) = value.value else {
                return Err(Error::new_spanned(
                    value.value,
                    "значение `fallback` должно быть путём",
                ));
            };
            fallback = Some(path.path);
        }
        Ok(Self { fallback })
    }
}
