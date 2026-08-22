use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Error, Expr, Fields, ItemEnum, LitStr, Meta, Path, Result, Token, Type,
    parenthesized,
};

use crate::model::{Declaration, Destination, Field, Question};

pub(crate) fn expand(arguments: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let context = parse_context(arguments)?;
    let mut item = syn::parse2::<ItemEnum>(input)?;
    let enum_ident = item.ident.clone();
    let args_ident = format_ident!("{}Args", enum_ident);
    let visibility = item.vis.clone();
    let mut structures = Vec::new();
    let mut argument_variants = Vec::new();
    let mut fill_arms = Vec::new();

    for variant in &mut item.variants {
        let about = parse_about(&variant.attrs, &variant.ident)?;
        variant
            .attrs
            .retain(|attribute| !attribute.path().is_ident("hill"));
        let Fields::Named(fields) = &mut variant.fields else {
            return Err(Error::new_spanned(
                &variant.fields,
                format!(
                    "вариант `{}` должен содержать именованные поля",
                    variant.ident
                ),
            ));
        };
        let structure_ident = format_ident!("{}Args", variant.ident);
        let variant_ident = &variant.ident;
        let mut generated_fields = Vec::new();

        for (position, field) in fields.named.iter_mut().enumerate() {
            let ident = field
                .ident
                .clone()
                .ok_or_else(|| Error::new(field.ty.span(), "ожидалось именованное поле"))?;
            let parsed = parse_command_field(field, &ident)?;
            let ty = field.ty.clone();
            let argument_ty = match (&parsed.arguments, &parsed.resolver, &parsed.question) {
                (Some(arguments), None, None) => arguments.clone(),
                (None, Some(_), None) | (None, None, Some(Question::Ask(_))) => {
                    syn::parse2(quote_spanned!(ty.span()=> Option<#ty>))?
                }
                (None, None, Some(Question::Keep(_))) => {
                    let Some(inner) = crate::parser::option_inner_type(&ty) else {
                        return Err(Error::new_spanned(
                            &ty,
                            format!("поле `{ident}` с пометкой `keep` должно иметь тип Option<T>"),
                        ));
                    };
                    syn::parse2(quote_spanned!(ty.span()=> Option<#inner>))?
                }
                (None, None, None) => {
                    if let Some(inner) = crate::parser::option_inner_type(&ty) {
                        syn::parse2(quote_spanned!(ty.span()=> Option<#inner>))?
                    } else {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "поле `{ident}` должно содержать пометку `ask`, `keep`, `with` или `args`"
                            ),
                        ));
                    }
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("поле `{ident}` содержит конфликтующие пометки"),
                    ));
                }
            };
            let mut clap_attributes = parsed.clap_attributes;
            if parsed.arguments.is_some() {
                clap_attributes.extend(Attribute::parse_outer.parse2(quote_spanned! {
                    ident.span()=> #[command(flatten)]
                })?);
            }
            generated_fields.push(Field {
                visibility: field.vis.clone(),
                ident,
                ty: argument_ty,
                position,
                clap_attributes,
                markers: Vec::new(),
                question: parsed.question,
                with_context: parsed.with_context,
                resolver: parsed.resolver,
                resolver_message: parsed.resolver_message,
            });
            field
                .attrs
                .retain(|attribute| !attribute.path().is_ident("hill"));
        }

        structures.push(crate::generate::generate(Declaration {
            visibility: visibility.clone(),
            ident: structure_ident.clone(),
            generics: syn::Generics::default(),
            destination: Destination {
                initializer: syn::parse2(quote!(#enum_ident::#variant_ident))?,
                result: syn::parse2(quote!(#enum_ident))?,
            },
            context: Some(context.clone()),
            clap_attributes: Vec::new(),
            fields: generated_fields,
        })?);
        argument_variants.push(quote_spanned! {variant_ident.span()=>
            #[command(about = #about)]
            #variant_ident(#structure_ident)
        });
        fill_arms.push(quote_spanned! {variant_ident.span()=>
            #args_ident::#variant_ident(args) => args.resolve(ctx).await
        });
    }

    Ok(quote! {
        #item

        #(#structures)*

        #[derive(clap::Subcommand)]
        #visibility enum #args_ident {
            #(#argument_variants,)*
        }

        #visibility async fn fill(
            args: #args_ident,
            ctx: &#context,
        ) -> ::command_hills::Result<#enum_ident> {
            match args {
                #(#fill_arms,)*
            }
        }
    })
}

fn parse_context(arguments: TokenStream) -> Result<Path> {
    let options = syn::parse2::<CommandsOptions>(arguments.clone())?;
    options.context.ok_or_else(|| {
        Error::new(
            arguments.span(),
            "для commands не указан параметр `context`",
        )
    })
}

struct CommandsOptions {
    context: Option<Path>,
}

impl syn::parse::Parse for CommandsOptions {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        let entries = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut context = None;

        for entry in entries {
            let Meta::NameValue(value) = entry else {
                return Err(Error::new_spanned(
                    entry,
                    "ожидался параметр в форме `context = Тип`",
                ));
            };
            let Some(name) = value.path.get_ident() else {
                return Err(Error::new_spanned(value.path, "ожидалось имя параметра"));
            };
            if name != "context" {
                return Err(Error::new(
                    name.span(),
                    format!("неизвестный параметр `{name}`"),
                ));
            }
            if context.is_some() {
                return Err(Error::new(
                    name.span(),
                    "параметр `context` указан несколько раз",
                ));
            }
            let Expr::Path(path) = value.value else {
                return Err(Error::new_spanned(
                    value.value,
                    "значение параметра `context` должно быть путём",
                ));
            };
            context = Some(path.path);
        }

        Ok(Self { context })
    }
}

fn parse_about(attributes: &[Attribute], variant: &syn::Ident) -> Result<LitStr> {
    let mut about = None;
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("hill"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("about") {
                return Err(meta.error(format!("неизвестная пометка варианта `{variant}`")));
            }
            if about.is_some() {
                return Err(meta.error(format!(
                    "пометка `about` варианта `{variant}` указана несколько раз"
                )));
            }
            about = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    about.ok_or_else(|| {
        Error::new(
            variant.span(),
            format!("у варианта `{variant}` отсутствует пометка `about`"),
        )
    })
}

fn parse_command_field(field: &syn::Field, ident: &syn::Ident) -> Result<CommandField> {
    let mut resolver = None;
    let mut question = None;
    let mut arguments = None;
    let mut with_context = false;
    let mut resolver_message = None;
    let mut clap_attributes = Vec::new();
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("hill"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("with") {
                if resolver.is_some() {
                    return Err(meta.error(format!(
                        "пометка `with` поля `{ident}` указана несколько раз"
                    )));
                }
                resolver = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("message") {
                if resolver_message.is_some() {
                    return Err(meta.error(format!(
                        "параметр `message` поля `{ident}` указан несколько раз"
                    )));
                }
                resolver_message = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("args") {
                if arguments.is_some() {
                    return Err(meta.error(format!(
                        "пометка `args` поля `{ident}` указана несколько раз"
                    )));
                }
                arguments = Some(meta.value()?.parse::<Type>()?);
                return Ok(());
            }
            if meta.path.is_ident("ctx") {
                if with_context {
                    return Err(meta.error(format!(
                        "пометка `ctx` поля `{ident}` указана несколько раз"
                    )));
                }
                with_context = true;
                return Ok(());
            }
            if meta.path.is_ident("ask") || meta.path.is_ident("keep") {
                let kind = if meta.path.is_ident("ask") {
                    "ask"
                } else {
                    "keep"
                };
                if question.is_some() {
                    return Err(meta.error(format!(
                        "поле `{ident}` не может сочетать пометки `ask` и `keep`"
                    )));
                }
                let message = meta.value()?.parse()?;
                question = Some(if kind == "ask" {
                    Question::Ask(message)
                } else {
                    Question::Keep(message)
                });
                return Ok(());
            }
            let attribute_ident = if meta.path.is_ident("arg") {
                quote!(arg)
            } else if meta.path.is_ident("command") {
                quote!(command)
            } else {
                return Err(meta.error(format!("неизвестная пометка поля `{ident}`")));
            };
            let content;
            parenthesized!(content in meta.input);
            let tokens: TokenStream = content.parse()?;
            clap_attributes.extend(Attribute::parse_outer.parse2(quote_spanned! {
                meta.path.span()=> #[#attribute_ident(#tokens)]
            })?);
            Ok(())
        })?;
    }
    if resolver.is_some() && question.is_some() {
        return Err(Error::new(
            ident.span(),
            format!("поле `{ident}` не может сочетать `with` с `ask` или `keep`"),
        ));
    }
    if arguments.is_some() && (resolver.is_some() || question.is_some()) {
        return Err(Error::new(
            ident.span(),
            format!("поле `{ident}` не может сочетать `args` с `ask`, `keep` или `with`"),
        ));
    }
    if with_context && arguments.is_none() {
        return Err(Error::new(
            ident.span(),
            format!("поле `{ident}` может содержать `ctx` только вместе с `args`"),
        ));
    }
    if resolver_message.is_some() && resolver.is_none() {
        return Err(Error::new(
            ident.span(),
            format!("поле `{ident}` может содержать `message` только вместе с `with`"),
        ));
    }
    Ok(CommandField {
        resolver,
        question,
        arguments,
        with_context,
        resolver_message,
        clap_attributes,
    })
}

struct CommandField {
    resolver: Option<Path>,
    question: Option<Question>,
    arguments: Option<Type>,
    with_context: bool,
    resolver_message: Option<LitStr>,
    clap_attributes: Vec<Attribute>,
}
