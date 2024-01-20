use super::{
    bindable::Definition,
    impl_locatable,
    location::{Locatable, TokenLocation},
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

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
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
    Array { size: u32, array_type: Box<Type> },
    Ptr(Box<Type>),
    Void,
}

impl_locatable!(Type);
