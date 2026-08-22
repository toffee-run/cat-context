use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Error, Expr, Field as SynField, Fields, GenericArgument, ItemStruct, Meta,
    PathArguments, Result, Token, Type,
};

use crate::model::{Declaration, Field, Marker, Question};

const CLAP_ATTRIBUTES: &[&str] = &["arg", "command"];
const EXCEPTION_MARKERS: &[&str] = &["Only"];

pub(crate) fn parse_declaration(arguments: TokenStream, input: TokenStream) -> Result<Declaration> {
    let item = syn::parse2::<ItemStruct>(input)?;
    let options = syn::parse2::<Options>(arguments)?;
    let target = options.target.ok_or_else(|| {
        Error::new(
            item.ident.span(),
            format!("для структуры `{}` не указан целевой тип", item.ident),
        )
    })?;
    let fields = parse_fields(&item)?;

    Ok(Declaration {
        visibility: item.vis,
        ident: item.ident,
        generics: item.generics,
        target,
        context: options.context,
        clap_attributes: clap_attributes(&item.attrs),
        fields,
    })
}

#[derive(Default)]
struct Options {
    target: Option<syn::Path>,
    context: Option<syn::Path>,
}

impl syn::parse::Parse for Options {
    fn parse(input: syn::parse::ParseStream<'_>) -> Result<Self> {
        let entries = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut options = Self::default();

        for entry in entries {
            let Meta::NameValue(value) = entry else {
                return Err(Error::new_spanned(
                    entry,
                    "ожидался параметр в форме `имя = Тип`",
                ));
            };
            let Some(name) = value.path.get_ident() else {
                return Err(Error::new_spanned(value.path, "ожидалось имя параметра"));
            };
            let Expr::Path(path) = value.value else {
                return Err(Error::new_spanned(
                    value.value,
                    format!("значение параметра `{name}` должно быть путём"),
                ));
            };

            match name.to_string().as_str() {
                "target" => set_once(&mut options.target, path.path, name)?,
                "context" => set_once(&mut options.context, path.path, name)?,
                _ => {
                    return Err(Error::new(
                        name.span(),
                        format!("неизвестный параметр `{name}`"),
                    ));
                }
            }
        }

        Ok(options)
    }
}

fn set_once(slot: &mut Option<syn::Path>, value: syn::Path, name: &syn::Ident) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(Error::new(
            name.span(),
            format!("параметр `{name}` указан несколько раз"),
        ));
    }

    Ok(())
}

fn parse_fields(item: &ItemStruct) -> Result<Vec<Field>> {
    let Fields::Named(fields) = &item.fields else {
        return Err(Error::new_spanned(
            &item.fields,
            format!(
                "структура `{}` должна содержать именованные поля",
                item.ident
            ),
        ));
    };

    fields
        .named
        .iter()
        .enumerate()
        .map(|(position, field)| parse_field(field, position))
        .collect()
}

fn parse_field(field: &SynField, position: usize) -> Result<Field> {
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| Error::new(field.ty.span(), "ожидалось именованное поле"))?;
    let markers = parse_markers(field, &ident)?;
    let (question, with_context) = parse_hill(field, &ident)?;

    Ok(Field {
        visibility: field.vis.clone(),
        ident,
        ty: field.ty.clone(),
        position,
        clap_attributes: clap_attributes(&field.attrs),
        markers,
        question,
        with_context,
    })
}

fn parse_hill(field: &SynField, field_ident: &syn::Ident) -> Result<(Option<Question>, bool)> {
    let mut question = None;
    let mut with_context = false;

    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("hill"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("ctx") {
                if with_context {
                    return Err(meta.error(format!(
                        "пометка `ctx` поля `{field_ident}` указана несколько раз"
                    )));
                }
                with_context = true;
                return Ok(());
            }

            let kind = if meta.path.is_ident("ask") {
                "ask"
            } else if meta.path.is_ident("keep") {
                "keep"
            } else {
                return Err(meta.error(format!("неизвестная пометка поля `{field_ident}`")));
            };
            let message = meta.value()?.parse::<syn::LitStr>()?;
            if question.is_some() {
                return Err(meta.error(format!(
                    "поле `{field_ident}` содержит несколько пометок вопроса"
                )));
            }
            question = Some(if kind == "ask" {
                Question::Ask(message)
            } else {
                Question::Keep(message)
            });
            Ok(())
        })?;
    }

    if question.is_some() && with_context {
        return Err(Error::new(
            field_ident.span(),
            format!("поле `{field_ident}` не может одновременно содержать вопрос и `ctx`"),
        ));
    }
    if question.is_some() && option_inner_type(&field.ty).is_none() {
        return Err(Error::new_spanned(
            &field.ty,
            format!("поле `{field_ident}` с вопросом должно иметь тип Option<T>"),
        ));
    }

    Ok((question, with_context))
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

fn parse_markers(field: &SynField, field_ident: &syn::Ident) -> Result<Vec<Marker>> {
    let Type::Path(type_path) = &field.ty else {
        return Ok(Vec::new());
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(Vec::new());
    };

    if !EXCEPTION_MARKERS
        .iter()
        .any(|marker| segment.ident == marker)
    {
        return Ok(Vec::new());
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(invalid_marker_error(field, field_ident, &segment.ident));
    };
    let mut type_arguments = arguments.args.iter().filter_map(|argument| {
        if let GenericArgument::Type(ty) = argument {
            Some(ty)
        } else {
            None
        }
    });
    let Some(inner_type) = type_arguments.next() else {
        return Err(invalid_marker_error(field, field_ident, &segment.ident));
    };

    if type_arguments.next().is_some() || arguments.args.len() != 1 {
        return Err(invalid_marker_error(field, field_ident, &segment.ident));
    }

    Ok(vec![Marker {
        ident: segment.ident.clone(),
        inner_type: inner_type.clone(),
    }])
}

fn invalid_marker_error(field: &SynField, field_ident: &syn::Ident, marker: &syn::Ident) -> Error {
    Error::new_spanned(
        &field.ty,
        format!("пометка `{marker}` поля `{field_ident}` должна содержать ровно один тип"),
    )
}

fn clap_attributes(attributes: &[Attribute]) -> Vec<Attribute> {
    attributes
        .iter()
        .filter(|attribute| {
            CLAP_ATTRIBUTES
                .iter()
                .any(|name| attribute.path().is_ident(name))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use quote::{ToTokens, quote};

    use super::*;

    fn parse(arguments: TokenStream, input: TokenStream) -> Result<Declaration> {
        parse_declaration(arguments, input)
    }

    #[test]
    fn parses_regular_field() {
        let declaration = parse(
            quote!(target = Action::Start, context = Docker),
            quote!(
                struct Start {
                    base: Option<Base>,
                }
            ),
        )
        .expect("обычное поле должно разбираться");

        assert_eq!(declaration.ident, "Start");
        assert_eq!(
            declaration.target.to_token_stream().to_string(),
            "Action :: Start"
        );
        assert_eq!(
            declaration
                .context
                .as_ref()
                .expect("контекст должен быть сохранён")
                .to_token_stream()
                .to_string(),
            "Docker"
        );
        assert_eq!(declaration.fields.len(), 1);
        assert_eq!(declaration.fields[0].ident, "base");
        assert_eq!(
            declaration.fields[0].ty.to_token_stream().to_string(),
            "Option < Base >"
        );
        assert_eq!(declaration.fields[0].position, 0);
    }

    #[test]
    fn preserves_clap_attributes() {
        let declaration = parse(
            quote!(target = Action::Start, context = Docker),
            quote!(
                #[command(rename_all = "kebab-case")]
                struct Start {
                    #[arg(long, value_parser = parse_base)]
                    #[command(flatten)]
                    base: Base,
                }
            ),
        )
        .expect("атрибуты clap должны разбираться");

        assert_eq!(declaration.clap_attributes.len(), 1);
        assert_eq!(
            declaration.clap_attributes[0].to_token_stream().to_string(),
            "# [command (rename_all = \"kebab-case\")]"
        );
        assert_eq!(declaration.fields[0].clap_attributes.len(), 2);
        assert_eq!(
            declaration.fields[0].clap_attributes[0]
                .to_token_stream()
                .to_string(),
            "# [arg (long , value_parser = parse_base)]"
        );
        assert_eq!(
            declaration.fields[0].clap_attributes[1]
                .to_token_stream()
                .to_string(),
            "# [command (flatten)]"
        );
    }

    #[test]
    fn recognizes_exception_marker() {
        let declaration = parse(
            quote!(target = Action::Restart, context = Docker),
            quote!(
                struct Restart {
                    no_save: Only<bool>,
                }
            ),
        )
        .expect("пометка исключения должна разбираться");

        assert_eq!(declaration.fields[0].markers.len(), 1);
        assert_eq!(declaration.fields[0].markers[0].ident, "Only");
        assert_eq!(
            declaration.fields[0].markers[0]
                .inner_type
                .to_token_stream()
                .to_string(),
            "bool"
        );
    }

    #[test]
    fn preserves_field_order() {
        let declaration = parse(
            quote!(target = Action::Restart, context = Docker),
            quote! {
                struct Restart {
                    container: String,
                    base: Option<Base>,
                    save: bool,
                }
            },
        )
        .expect("несколько полей должны разбираться");

        let fields = declaration
            .fields
            .iter()
            .map(|field| (field.ident.to_string(), field.position))
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            vec![
                ("container".to_owned(), 0),
                ("base".to_owned(), 1),
                ("save".to_owned(), 2)
            ]
        );
    }

    #[test]
    fn rejects_missing_target() {
        let error = parse(
            quote!(context = Docker),
            quote!(
                struct Restart {
                    base: Option<Base>,
                }
            ),
        )
        .err()
        .expect("отсутствующий целевой тип должен давать ошибку");

        assert!(error.to_string().contains("Restart"));
        assert!(error.to_string().contains("целевой тип"));
    }

    #[test]
    fn marker_error_names_problematic_field() {
        let error = parse(
            quote!(target = Action::Restart, context = Docker),
            quote!(
                struct Restart {
                    no_save: Only,
                }
            ),
        )
        .err()
        .expect("некорректная пометка должна давать ошибку");

        assert!(error.to_string().contains("no_save"));
    }

    #[test]
    fn rejects_question_with_context() {
        let error = parse(
            quote!(target = Action::Start, context = Docker),
            quote! {
                struct Start {
                    #[hill(ask = "Base", ctx)]
                    base: Option<Base>,
                }
            },
        )
        .err()
        .expect("вопрос вместе с контекстом должен давать ошибку");

        assert!(error.to_string().contains("base"));
    }
}
