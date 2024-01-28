use std::ops::Deref;

use crate::ast::{
    self, ArrayAccess, ArrayInitializer, Assignment, BinaryOperation, Call, Expression,
    FunctionStatement, LetStatement, Literal, StructStatement,
};

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
    /// This type is for int literal and is supposed to be compatible with any
    /// integer like type (signed and unsigned).
    /// It is only used internaly in the ast.
    Int,
    Float,
    String,
    Bool,
    Struct {
        name: String,
        fields: Vec<FunctionParameter>,
    },
    Function {
        parameters: Vec<FunctionParameter>,
        return_type: Box<Type>,
    },
    Array {
        size: u32,
        array_type: Box<Type>,
    },
    Ptr(Box<Type>),
    Void,
    Null {
        concrete_type: Option<Box<Type>>,
    },
}

impl Type {
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            // `Int` must be compatible with itself to allow stuff like 1 + 1
            (Type::Int, Type::Int)
            | (Type::Int, Type::U8)
            | (Type::Int, Type::U16)
            | (Type::Int, Type::U32)
            | (Type::Int, Type::U64)
            | (Type::Int, Type::I8)
            | (Type::Int, Type::I16)
            | (Type::Int, Type::I32)
            | (Type::Int, Type::I64)
            | (Type::U8, Type::Int)
            | (Type::U16, Type::Int)
            | (Type::U32, Type::Int)
            | (Type::U64, Type::Int)
            | (Type::I8, Type::Int)
            | (Type::I16, Type::Int)
            | (Type::I32, Type::Int)
            | (Type::I64, Type::Int)
            | (Type::U8, Type::U8)
            | (Type::U16, Type::U16)
            | (Type::U32, Type::U32)
            | (Type::U64, Type::U64)
            | (Type::I8, Type::I8)
            | (Type::I16, Type::I16)
            | (Type::I32, Type::I32)
            | (Type::I64, Type::I64)
            | (Type::Bool, Type::Bool)
            | (Type::Void, Type::Void)
            | (Type::Ptr(_), Type::Null { .. })
            | (Type::String, Type::String) => true,
            (
                Type::Array {
                    size: lsize,
                    array_type: larray_type,
                },
                Type::Array {
                    size: rsize,
                    array_type: rarray_rtype,
                },
            ) => lsize == rsize && larray_type.is_compatible_with(rarray_rtype),
            (Type::Ptr(l), Type::Ptr(r)) => l.is_compatible_with(r),
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::Int
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }

    /// Return array base type.
    /// # Panics
    /// This method panic if the type is not an array
    pub fn get_base_array_type(&self) -> &Type {
        match self {
            Type::Array { array_type, .. } => array_type,
            _ => panic!("Call get_base_array_type on a type that is not an array!"),
        }
    }
}

impl From<ast::TypeKind> for Type {
    fn from(value: ast::TypeKind) -> Self {
        match value {
            ast::TypeKind::U8 => Type::U8,
            ast::TypeKind::U16 => Type::U16,
            ast::TypeKind::U32 => Type::U32,
            ast::TypeKind::U64 => Type::U64,
            ast::TypeKind::I8 => Type::I8,
            ast::TypeKind::I16 => Type::I16,
            ast::TypeKind::I32 => Type::I32,
            ast::TypeKind::I64 => Type::I64,
            ast::TypeKind::String => Type::String,
            ast::TypeKind::Bool => Type::Bool,
            ast::TypeKind::Float => Type::Float,
            ast::TypeKind::Identifier(name) => Type::Struct {
                name,
                fields: Vec::new(),
            },
            ast::TypeKind::Void => Type::Void,
            ast::TypeKind::Array { size, array_type } => Type::Array {
                size,
                array_type: Box::new(array_type.kind.into()),
            },
            ast::TypeKind::Ptr(ptr) => Type::Ptr(Box::new(ptr.deref().to_owned().kind.into())),
            ast::TypeKind::Null { .. } => Type::Null {
                concrete_type: None,
            },
        }
    }
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

impl_typables!(
    Assignment,
    BinaryOperation,
    Call,
    FunctionStatement,
    LetStatement,
    Literal,
    ArrayAccess,
    ArrayInitializer,
    StructStatement
);

impl Typable for Expression {
    fn get_type(&self) -> &Type {
        match self {
            Expression::Group(g) => g.get_type(),
            Expression::BinaryOperation(bo) => bo.get_type(),
            Expression::Literal(l) => l.get_type(),
            Expression::Call(c) => c.get_type(),
            Expression::Assignment(a) => a.get_type(),
            Expression::ArrayInitializer(a) => a.get_type(),
            Expression::AddrOf(_) => todo!(),
            Expression::Deref(_) => todo!(),
        }
    }

    fn set_type(&mut self, _: Type) {
        unreachable!("Cannot set type to an expression directly");
    }
}
