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
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
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
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: String,
    pub arguments: Vec<Expression>,
    location: TokenLocation,
    pub(crate) definition: Option<Definition>,
}

impl Call {
    pub fn new(tk_begin: usize, tk_end: usize, callee: String, arguments: Vec<Expression>) -> Self {
        Self {
            callee,
            arguments,
            location: TokenLocation::new(tk_begin, tk_end),
            definition: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub left: Box<Expression>,
    pub right: Option<Box<Expression>>,
    pub op: OpType,
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

impl_locatable!(Assignment, Call, Literal, BinaryOperation);
