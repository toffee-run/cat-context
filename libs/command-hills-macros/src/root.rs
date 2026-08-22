use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Error, Expr, Fields, ItemStruct, LitStr, Meta, Path, Result, Token, Type,
    parenthesized,
};

pub(crate) fn expand(arguments: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let options = syn::parse2::<Options>(arguments)?;
    let action = options
        .action
        .ok_or_else(|| Error::new(options.span, "для root не указан параметр `action`"))?;
    let ask = options
        .ask
        .ok_or_else(|| Error::new(options.span, "для root не указан параметр `ask`"))?;
    let mut item = syn::parse2::<ItemStruct>(input)?;
    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            &item.fields,
            format!(
                "структура `{}` должна содержать именованные поля",
                item.ident
            ),
        ));
    };
    if fields
        .named
        .iter()
        .any(|field| field.ident.as_ref().is_some_and(|ident| ident == "action"))
    {
        return Err(Error::new(
            item.ident.span(),
            format!(
                "структура `{}` не должна объявлять поле `action`",
                item.ident
            ),
        ));
    }

    let visibility = item.vis.clone();
    let command_ident = item.ident.clone();
    let mut action_args = action.clone();
    let Some(action_segment) = action_args.segments.last_mut() else {
        return Err(Error::new_spanned(
            action,
            "параметр `action` должен быть путём",
        ));
    };
    action_segment.ident = format_ident!("{}Args", action_segment.ident);
    let cli_attributes = item
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("command"))
        .cloned()
        .collect::<Vec<_>>();
    item.attrs.retain(|attribute| {
        !attribute.path().is_ident("command") && !attribute.path().is_ident("hill")
    });

    let mut parsed_fields = Vec::new();
    let mut context = None;
    for field in &mut fields.named {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| Error::new(field.ty.span(), "ожидалось именованное поле"))?;
        let parsed = parse_field(field, &ident)?;
        if parsed.context && context.replace(ident.clone()).is_some() {
            return Err(Error::new(
                ident.span(),
                format!("поле `{ident}` содержит лишнюю пометку `context`"),
            ));
        }
        parsed_fields.push(RootField {
            visibility: field.vis.clone(),
            ident,
            ty: field.ty.clone(),
            resolver: parsed.resolver,
            clap_attributes: parsed.clap_attributes,
        });
        field
            .attrs
            .retain(|attribute| !attribute.path().is_ident("hill"));
    }
    let context = context.ok_or_else(|| {
        let ident = parsed_fields
            .first()
            .map_or_else(|| item.ident.clone(), |field| field.ident.clone());
        Error::new(
            ident.span(),
            format!(
                "поле `{ident}` должно содержать пометку `context`, чтобы передать контекст в `fill`"
            ),
        )
    })?;

    let domain_action = syn::Field::parse_named.parse2(quote!(#visibility action: #action))?;
    fields.named.push(domain_action);
    let cli_fields = parsed_fields.iter().map(|field| {
        let visibility = &field.visibility;
        let ident = &field.ident;
        let ty = &field.ty;
        let attributes = &field.clap_attributes;
        quote_spanned! {ident.span()=>
            #(#attributes)*
            #visibility #ident: Option<#ty>
        }
    });
    let resolved_fields = parsed_fields.iter().map(|field| {
        let ident = &field.ident;
        let resolver = &field.resolver;
        quote_spanned! {ident.span()=> let #ident = #resolver(self.#ident);}
    });
    let initialized_fields = parsed_fields.iter().map(|field| &field.ident);
    let action_visibility = &visibility;

    Ok(quote! {
        #item

        #[derive(clap::Parser)]
        #(#cli_attributes)*
        #visibility struct Cli {
            #(#cli_fields,)*
            #[command(subcommand)]
            #action_visibility action: Option<#action_args>,
        }

        impl Cli {
            #visibility async fn resolve(self) -> ::command_hills::Result<#command_ident> {
                #(#resolved_fields)*
                let action_args = match self.action {
                    Some(action) => action,
                    None => {
                        let chosen = ::command_hills::__private::ask_subcommand::<Cli>(#ask)?;
                        let parsed = ::command_hills::__private::clap_result(
                            <Cli as clap::Parser>::try_parse_from([
                                ::std::string::String::from("command-hills"),
                                chosen,
                            ]),
                        )?;
                        ::command_hills::__private::required_subcommand(parsed.action)?
                    }
                };
                let action = fill(action_args, &#context).await?;
                Ok(#command_ident {
                    #(#initialized_fields,)*
                    action,
                })
            }
        }

        impl #command_ident {
            #visibility async fn parse() -> ::command_hills::Result<Self> {
                <Cli as clap::Parser>::parse().resolve().await
            }
        }
    })
}

struct RootField {
    visibility: syn::Visibility,
    ident: syn::Ident,
    ty: Type,
    resolver: Path,
    clap_attributes: Vec<Attribute>,
}

struct ParsedField {
    context: bool,
    resolver: Path,
    clap_attributes: Vec<Attribute>,
}

fn parse_field(field: &syn::Field, ident: &syn::Ident) -> Result<ParsedField> {
    let mut context = false;
    let mut resolver = None;
    let mut clap_attributes = Vec::new();
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("hill"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("context") {
                if context {
                    return Err(meta.error(format!(
                        "пометка `context` поля `{ident}` указана несколько раз"
                    )));
                }
                context = true;
                return Ok(());
            }
            if meta.path.is_ident("with") {
                if resolver.is_some() {
                    return Err(meta.error(format!(
                        "пометка `with` поля `{ident}` указана несколько раз"
                    )));
                }
                resolver = Some(meta.value()?.parse()?);
                return Ok(());
            }
            let attribute_ident = if meta.path.is_ident("arg") {
                quote!(arg)
            } else if meta.path.is_ident("command") {
                quote!(command)
            } else {
                return Err(meta.error(format!(
                    "неизвестная пометка поля `{ident}`; допустимы: `context`, `with`, `arg`, `command`"
                )));
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
    let resolver = resolver.ok_or_else(|| {
        Error::new(
            ident.span(),
            format!("поле `{ident}` должно содержать пометку `with`"),
        )
    })?;
    Ok(ParsedField {
        context,
        resolver,
        clap_attributes,
    })
}

struct Options {
    action: Option<Path>,
    ask: Option<LitStr>,
    span: proc_macro2::Span,
}

impl syn::parse::Parse for Options {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        let entries = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut options = Self {
            action: None,
            ask: None,
            span: input.span(),
        };
        for entry in entries {
            let Meta::NameValue(value) = entry else {
                return Err(Error::new_spanned(
                    entry,
                    "ожидался параметр в форме `имя = значение`",
                ));
            };
            let Some(name) = value.path.get_ident() else {
                return Err(Error::new_spanned(value.path, "ожидалось имя параметра"));
            };
            match name.to_string().as_str() {
                "action" => {
                    if options.action.is_some() {
                        return Err(Error::new(
                            name.span(),
                            "параметр `action` указан несколько раз",
                        ));
                    }
                    let Expr::Path(path) = value.value else {
                        return Err(Error::new_spanned(
                            value.value,
                            "значение параметра `action` должно быть путём",
                        ));
                    };
                    options.action = Some(path.path);
                }
                "ask" => {
                    if options.ask.is_some() {
                        return Err(Error::new(
                            name.span(),
                            "параметр `ask` указан несколько раз",
                        ));
                    }
                    let Expr::Lit(literal) = value.value else {
                        return Err(Error::new_spanned(
                            value.value,
                            "значение параметра `ask` должно быть строкой",
                        ));
                    };
                    let syn::Lit::Str(message) = literal.lit else {
                        return Err(Error::new_spanned(
                            literal,
                            "значение параметра `ask` должно быть строкой",
                        ));
                    };
                    options.ask = Some(message);
                }
                _ => {
                    return Err(Error::new(
                        name.span(),
                        format!("неизвестный параметр `{name}`; допустимы `action` и `ask`"),
                    ));
                }
            }
        }
        Ok(options)
    }
}
