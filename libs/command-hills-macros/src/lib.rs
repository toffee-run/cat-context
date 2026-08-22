use proc_macro::TokenStream;

mod clean;
mod commands;
mod generate;
mod model;
mod parser;
mod root;

#[proc_macro_attribute]
pub fn root(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let original = input.clone().into();
    match root::expand(arguments.into(), input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => clean::preserve_item(error, original).into(),
    }
}

#[proc_macro_attribute]
pub fn commands(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let original = input.clone().into();
    match commands::expand(arguments.into(), input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => clean::preserve_item(error, original).into(),
    }
}

#[proc_macro_attribute]
pub fn fill(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let original = input.clone().into();
    match parser::parse_declaration(arguments.into(), input.into()).and_then(generate::generate) {
        Ok(tokens) => tokens.into(),
        Err(error) => clean::preserve_item(error, original).into(),
    }
}
