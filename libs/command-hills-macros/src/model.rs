use syn::{Attribute, Generics, Ident, Path, Type, Visibility};

pub(crate) struct Declaration {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) target: Path,
    pub(crate) context: Path,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) fields: Vec<Field>,
}

pub(crate) struct Field {
    pub(crate) visibility: Visibility,
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) position: usize,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) markers: Vec<Marker>,
}

pub(crate) struct Marker {
    pub(crate) ident: Ident,
    pub(crate) inner_type: Type,
}
