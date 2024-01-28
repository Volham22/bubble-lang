use crate::type_system;

use super::{
    bindable::Definition,
    impl_locatable,
    location::{Locatable, TokenLocation},
};

#[derive(Debug, Clone)]
pub enum Expression {
    Group(Box<Expression>),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
    Call(Call),
    Assignment(Assignment),
    ArrayInitializer(ArrayInitializer),
    AddrOf(AddrOf),
    Deref(Deref),
}

impl Expression {
    pub fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal(_))
    }
}

#[derive(Debug, Clone)]
pub struct AddrOf {
    pub expr: Box<Expression>,
    location: TokenLocation,
}

impl AddrOf {
    pub fn new(tk_begin: usize, tk_end: usize, expr: Box<Expression>) -> Self {
        Self {
            expr,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Deref {
    pub expr: Box<Expression>,
    location: TokenLocation,
}

impl Deref {
    pub fn new(tk_begin: usize, tk_end: usize, expr: Box<Expression>) -> Self {
        Self {
            expr,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Null {
    location: TokenLocation,
}

impl Null {
    pub fn new(tk_begin: usize, tk_end: usize) -> Self {
        Self {
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub(crate) ty: Option<type_system::Type>,
    location: TokenLocation,
}

impl Assignment {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        left: Box<Expression>,
        right: Box<Expression>,
    ) -> Self {
        Self {
            left,
            right,
            ty: None,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: String,
    pub arguments: Vec<Expression>,
    location: TokenLocation,
    pub(crate) ty: Option<type_system::Type>,
    pub(crate) definition: Option<Definition>,
}

impl Call {
    pub fn new(tk_begin: usize, tk_end: usize, callee: String, arguments: Vec<Expression>) -> Self {
        Self {
            callee,
            arguments,
            location: TokenLocation::new(tk_begin, tk_end),
            ty: None,
            definition: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub left: Box<Expression>,
    pub right: Option<Box<Expression>>,
    pub op: OpType,
    pub(crate) ty: Option<type_system::Type>,
    location: TokenLocation,
}

impl BinaryOperation {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        left: Box<Expression>,
        op: OpType,
        right: Option<Box<Expression>>,
    ) -> Self {
        Self {
            left,
            right,
            op,
            ty: None,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub literal_type: LiteralType,
    pub(crate) definition: Option<Definition>,
    pub(crate) ty: Option<type_system::Type>,
    location: TokenLocation,
}

impl Literal {
    pub fn new(tk_begin: usize, tk_end: usize, literal_type: LiteralType) -> Self {
        Self {
            literal_type,
            location: TokenLocation::new(tk_begin, tk_end),
            definition: None,
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LiteralType {
    True,
    False,
    Integer(i64),
    Float(f64),
    Identifier(String),
    ArrayAccess(ArrayAccess),
    String(String),
    Null(Null),
}

#[derive(Debug, Copy, Clone)]
pub enum OpType {
    And,
    Different,
    Divide,
    Equal,
    Less,
    LessEqual,
    Minus,
    Modulo,
    More,
    MoreEqual,
    Multiply,
    Not,
    Or,
    Plus,
}

#[derive(Debug, Clone)]
pub struct ArrayAccess {
    pub identifier: Box<Expression>,
    pub index: Box<Expression>,
    pub(crate) definition: Option<Definition>,
    pub(crate) ty: Option<type_system::Type>,
    location: TokenLocation,
}

impl ArrayAccess {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        identifier: Box<Expression>,
        index: Box<Expression>,
    ) -> Self {
        Self {
            identifier,
            index,
            location: TokenLocation::new(tk_begin, tk_end),
            definition: None,
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayInitializer {
    pub values: Vec<Box<Expression>>,
    pub(crate) ty: Option<type_system::Type>,
    location: TokenLocation,
}

impl ArrayInitializer {
    pub fn new(tk_begin: usize, tk_end: usize, values: Vec<Box<Expression>>) -> Self {
        Self {
            values,
            location: TokenLocation::new(tk_begin, tk_end),
            ty: None,
        }
    }
}

impl_locatable!(
    AddrOf,
    ArrayAccess,
    ArrayInitializer,
    Assignment,
    BinaryOperation,
    Call,
    Literal,
    Null
);
