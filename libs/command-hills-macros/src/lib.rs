use proc_macro::TokenStream;

mod generate;
mod model;
mod parser;

#[proc_macro_attribute]
pub fn fill(arguments: TokenStream, input: TokenStream) -> TokenStream {
    match parser::parse_declaration(arguments.into(), input.into()).and_then(generate::generate) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
