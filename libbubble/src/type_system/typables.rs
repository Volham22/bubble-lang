use crate::ast::{FunctionStatement, LetStatement, Literal, StructStatement};

pub type FunctionParameter = (Type, String);

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
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
    Struct(String),
    Function {
        parameters: Vec<FunctionParameter>,
        return_type: Box<Type>,
    },
    Void,
}

pub trait Typable {
    fn get_type(&self) -> &Type;
    fn set_type(&mut self, ty: Type);
}

macro_rules! impl_typables {
    ( $( $name:ty ),* ) => {
        $(
            impl Typable for $name {
                fn get_type(&self) -> &Type {
                    self.ty.as_ref().expect("Access type on untyped node!")
                }

                fn set_type(&mut self, ty: Type) {
                    self.ty = Some(ty);
                }
            }
        )*
    };
}

impl_typables!(FunctionStatement, LetStatement, StructStatement, Literal);
