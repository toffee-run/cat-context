use syn::{Attribute, Generics, Ident, Path, Type, Visibility};

pub(crate) struct Declaration {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) destination: Destination,
    pub(crate) context: Option<Path>,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) fields: Vec<Field>,
}

pub(crate) struct Destination {
    pub(crate) initializer: Path,
    pub(crate) result: Path,
}

pub(crate) struct Field {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) position: usize,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) markers: Vec<Marker>,
    pub(crate) question: Option<Question>,
    pub(crate) with_context: bool,
    pub(crate) resolver: Option<Path>,
    pub(crate) resolver_message: Option<syn::LitStr>,
}

pub(crate) enum Question {
    Ask(syn::LitStr),
    Keep(syn::LitStr),
}

pub(crate) struct Marker {
    pub(crate) ident: Ident,
    pub(crate) inner_type: Type,
}
