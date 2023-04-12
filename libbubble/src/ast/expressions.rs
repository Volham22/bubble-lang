use super::location::{Locatable, TokenLocation};

#[derive(Debug)]
pub enum Expression {
    Group(Box<Expression>),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
}

#[derive(Debug)]
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

impl Locatable for BinaryOperation {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct Literal {
    pub literal_type: LiteralType,
    location: TokenLocation,
}

impl Literal {
    pub fn new(tk_begin: usize, tk_end: usize, literal_type: LiteralType) -> Self {
        Self {
            literal_type,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

impl Locatable for Literal {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub enum LiteralType {
    True,
    False,
    Integer(i64),
    Float(f64),
    Identifier(String),
}

#[derive(Debug)]
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
