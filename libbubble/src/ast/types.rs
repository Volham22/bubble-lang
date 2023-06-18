use super::{
    bindable::Definition,
    location::{Locatable, TokenLocation},
    visitor::Visitor,
    MutableVisitor,
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

    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_type(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_type(self)
    }
}

impl Locatable for Type {
    fn get_location(&self) -> &TokenLocation {
        &self.location
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
    Identifier(String),
    Void,
}

impl TypeKind {
    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_type_kind(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_type_kind(self)
    }
}
