use super::{
    bindable::Definition,
    location::{Locatable, TokenLocation}, impl_locatable,
};

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    location: TokenLocation,
    pub(crate) definition: Option<Definition>,
}

impl Type {
    pub fn new(tk_begin: usize, tk_end: usize, kind: TypeKind) -> Self {
        Self {
            kind,
            location: TokenLocation::new(tk_begin, tk_end),
            definition: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    String,
    Bool,
    Float,
    Identifier(String),
    Void,
}

impl_locatable!(Type);
