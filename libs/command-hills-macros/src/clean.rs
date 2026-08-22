use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Fields, Item};

pub(crate) fn preserve_item(error: syn::Error, input: TokenStream) -> TokenStream {
    let compile_error = error.into_compile_error();
    let Ok(mut item) = syn::parse2::<Item>(input) else {
        return compile_error;
    };
    clean_item(&mut item);
    quote! {
        #item
        #compile_error
    }
}

fn clean_item(item: &mut Item) {
    match item {
        Item::Enum(item) => {
            clean_attributes(&mut item.attrs);
            for variant in &mut item.variants {
                clean_attributes(&mut variant.attrs);
                clean_fields(&mut variant.fields);
            }
        }
        Item::Struct(item) => {
            clean_attributes(&mut item.attrs);
            clean_fields(&mut item.fields);
        }
        Item::Union(item) => {
            clean_attributes(&mut item.attrs);
            for field in &mut item.fields.named {
                clean_attributes(&mut field.attrs);
            }
        }
        _ => {}
    }
}

fn clean_fields(fields: &mut Fields) {
    for field in fields {
        clean_attributes(&mut field.attrs);
    }
}

fn clean_attributes(attributes: &mut Vec<Attribute>) {
    attributes.retain(|attribute| {
        !["hill", "arg", "command", "group", "value"]
            .iter()
            .any(|name| attribute.path().is_ident(name))
    });
}
