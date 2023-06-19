use super::{
    bindable::Definition,
    location::{Locatable, TokenLocation},
    visitor::Visitor,
    MutableVisitor,
};

#[derive(Debug, Clone)]
pub enum Expression {
    Group(Box<Expression>),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
    Call(Call),
    Assignment(Assignment),
}

impl Expression {
    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_expression(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_expression(self)
    }
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

    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_assignment(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_assignment(self)
    }
}

impl Locatable for Assignment {
    fn get_location(&self) -> &TokenLocation {
        &self.location
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

    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_call(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_call(self)
    }
}

impl Locatable for Call {
    fn get_location(&self) -> &TokenLocation {
        &self.location
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

    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_binary_operation(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_binary_operation(self)
    }
}

impl Locatable for BinaryOperation {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub literal_type: LiteralType,
    location: TokenLocation,
    pub(crate) definition: Option<Definition>,
}

impl Literal {
    pub fn new(tk_begin: usize, tk_end: usize, literal_type: LiteralType) -> Self {
        Self {
            literal_type,
            location: TokenLocation::new(tk_begin, tk_end),
            definition: None,
        }
    }

    pub fn accept<T, E>(&self, v: &mut T) -> Result<(), E>
    where
        T: Visitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_literal(self)
    }

    pub fn accept_mut<T, E>(&mut self, v: &mut T) -> Result<(), E>
    where
        T: MutableVisitor<E> + ?Sized,
        E: std::error::Error,
    {
        v.visit_literal(self)
    }
}

impl Locatable for Literal {
    fn get_location(&self) -> &TokenLocation {
        &self.location
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

#[derive(Debug, Clone)]
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
