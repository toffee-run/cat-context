use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Error, Expr, Field as SynField, Fields, GenericArgument, ItemStruct, Meta,
    PathArguments, Result, Token, Type,
    parse::{Parse, ParseStream},
};

use crate::model::{Declaration, Field, Marker};

const DECLARATION_ATTRIBUTE: &str = "command_hill";
const CLAP_ATTRIBUTES: &[&str] = &["arg", "command"];
const EXCEPTION_MARKERS: &[&str] = &["Only"];

impl Parse for Declaration {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let item = input.parse::<ItemStruct>()?;
        let target = parse_target(&item)?;
        let clap_attributes = clap_attributes(&item.attrs);
        let fields = parse_fields(&item)?;

        Ok(Self {
            ident: item.ident,
            target,
            clap_attributes,
            fields,
        })
    }
}

fn parse_target(item: &ItemStruct) -> Result<syn::Path> {
    let mut target = None;

    for attribute in item
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(DECLARATION_ATTRIBUTE))
    {
        let entries = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

        for entry in entries {
            if let Meta::NameValue(value) = entry
                && value.path.is_ident("target")
            {
                let Expr::Path(path) = value.value else {
                    return Err(Error::new_spanned(
                        value.value,
                        format!("целевой тип структуры `{}` должен быть путём", item.ident),
                    ));
                };

                if target.replace(path.path).is_some() {
                    return Err(Error::new_spanned(
                        value.path,
                        format!(
                            "целевой тип структуры `{}` указан несколько раз",
                            item.ident
                        ),
                    ));
                }
            }
        }
    }

    target.ok_or_else(|| {
        Error::new(
            item.ident.span(),
            format!("для структуры `{}` не указан целевой тип", item.ident),
        )
    })
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

    Ok(Field {
        ident,
        ty: field.ty.clone(),
        position,
        clap_attributes: clap_attributes(&field.attrs),
        markers,
    })
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

    if type_arguments.next().is_none()
        || type_arguments.next().is_some()
        || arguments.args.len() != 1
    {
        return Err(invalid_marker_error(field, field_ident, &segment.ident));
    }

    Ok(vec![Marker {
        ident: segment.ident.clone(),
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
    use quote::ToTokens;

    use super::*;

    #[test]
    fn parses_regular_field() {
        let declaration = syn::parse_str::<Declaration>(
            "#[command_hill(target = Action::Start)] struct Start { base: Option<Base> }",
        )
        .expect("обычное поле должно разбираться");

        assert_eq!(declaration.ident, "Start");
        assert_eq!(
            declaration.target.to_token_stream().to_string(),
            "Action :: Start"
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
        let declaration = syn::parse_str::<Declaration>(
            "#[command_hill(target = Action::Start)] #[command(rename_all = \"kebab-case\")] struct Start { #[arg(long, value_parser = parse_base)] #[command(flatten)] base: Base }",
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
        let declaration = syn::parse_str::<Declaration>(
            "#[command_hill(target = Action::Restart)] struct Restart { no_save: Only<bool> }",
        )
        .expect("пометка исключения должна разбираться");

        assert_eq!(declaration.fields[0].markers.len(), 1);
        assert_eq!(declaration.fields[0].markers[0].ident, "Only");
    }

    #[test]
    fn preserves_field_order() {
        let declaration = syn::parse_str::<Declaration>(
            "#[command_hill(target = Action::Restart)] struct Restart { container: String, base: Option<Base>, save: bool }",
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
        let error = syn::parse_str::<Declaration>("struct Restart { base: Option<Base> }")
            .err()
            .expect("отсутствующий целевой тип должен давать ошибку");

        assert!(error.to_string().contains("Restart"));
        assert!(error.to_string().contains("целевой тип"));
    }

    #[test]
    fn marker_error_names_problematic_field() {
        let error = syn::parse_str::<Declaration>(
            "#[command_hill(target = Action::Restart)] struct Restart { no_save: Only }",
        )
        .err()
        .expect("некорректная пометка должна давать ошибку");

        assert!(error.to_string().contains("no_save"));
    }
}
