use proc_macro::TokenStream;

mod commands;
mod generate;
mod model;
mod parser;

#[proc_macro_attribute]
pub fn commands(arguments: TokenStream, input: TokenStream) -> TokenStream {
    match commands::expand(arguments.into(), input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn fill(arguments: TokenStream, input: TokenStream) -> TokenStream {
    match parser::parse_declaration(arguments.into(), input.into()).and_then(generate::generate) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
