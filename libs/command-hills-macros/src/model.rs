use syn::{Attribute, Ident, Path, Type};

pub(crate) struct Declaration {
    pub(crate) ident: Ident,
    pub(crate) target: Path,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) fields: Vec<Field>,
}

pub(crate) struct Field {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) position: usize,
    pub(crate) clap_attributes: Vec<Attribute>,
    pub(crate) markers: Vec<Marker>,
}

pub(crate) struct Marker {
    pub(crate) ident: Ident,
}
