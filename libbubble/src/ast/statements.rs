use crate::type_system;

use super::{
    expressions::Expression,
    impl_locatable,
    location::{Locatable, TokenLocation},
    TypeKind,
};

#[derive(Debug, Clone)]
pub enum GlobalStatement {
    Function(FunctionStatement),
    Struct(StructStatement),
    Let(LetStatement),
}

pub type FunctionParameter = (TypeKind, String);

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeKind,
    pub is_extern: bool,
    pub body: Option<Statements>,
    location: TokenLocation,
    pub(crate) ty: Option<type_system::Type>,
}

impl FunctionStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: TypeKind,
        is_extern: bool,
        body: Option<Statements>,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            is_extern,
            body,
            location: TokenLocation::new(tk_begin, tk_end),
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub declaration_type: Option<TypeKind>,
    pub init_exp: Box<Expression>,
    location: TokenLocation,
    pub(crate) ty: Option<type_system::Type>,
}

impl LetStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        name: String,
        declaration_type: Option<TypeKind>,
        init_exp: Box<Expression>,
    ) -> Self {
        Self {
            name,
            declaration_type,
            init_exp,
            location: TokenLocation::new(tk_begin, tk_end),
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub exp: Option<Box<Expression>>,
    location: TokenLocation,
}

impl ReturnStatement {
    pub fn new(begin_tk: usize, end_tk: usize, exp: Option<Box<Expression>>) -> Self {
        Self {
            exp,
            location: TokenLocation::new(begin_tk, end_tk),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BreakStatement {
    location: TokenLocation,
}

impl BreakStatement {
    pub fn new(tk_begin: usize, tk_end: usize) -> Self {
        Self {
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    location: TokenLocation,
}

impl ContinueStatement {
    pub fn new(tk_begin: usize, tk_end: usize) -> Self {
        Self {
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructStatement {
    pub name: String,
    pub fields: Vec<FunctionParameter>,
    location: TokenLocation,
    pub(crate) ty: Option<type_system::Type>,
}

impl StructStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        name: String,
        fields: Vec<FunctionParameter>,
    ) -> Self {
        Self {
            name,
            fields,
            location: TokenLocation::new(tk_begin, tk_end),
            ty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statements {
    pub statements: Vec<Statement>,
    location: TokenLocation,
}

impl Statements {
    pub fn new(tk_begin: usize, tk_end: usize, statements: Vec<Statement>) -> Self {
        Self {
            statements,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }

    pub fn append_statement(&mut self, stmt: Statement) {
        self.statements.insert(0, stmt);
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    location: TokenLocation,
}

impl Statement {
    pub fn new(tk_begin: usize, tk_end: usize, kind: StatementKind) -> Self {
        Self {
            kind,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    If(IfStatement),
    Let(LetStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(ReturnStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Expression { expr: Box<Expression>, naked: bool },
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<Expression>,
    pub then_clause: Box<Statements>,
    pub else_clause: Option<Box<Statements>>,
    location: TokenLocation,
}

impl IfStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        condition: Box<Expression>,
        then_clause: Box<Statements>,
        else_clause: Option<Box<Statements>>,
    ) -> Self {
        Self {
            condition,
            then_clause,
            else_clause,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Box<Expression>,
    pub body: Box<Statements>,
    location: TokenLocation,
}

impl WhileStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        condition: Box<Expression>,
        body: Box<Statements>,
    ) -> Self {
        Self {
            condition,
            body,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub init_decl: LetStatement,
    pub continue_expression: Box<Expression>,
    pub modify_expression: Box<Expression>,
    pub body: Box<Statements>,
    location: TokenLocation,
}

impl ForStatement {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        init_decl: LetStatement,
        continue_expression: Box<Expression>,
        modify_expression: Box<Expression>,
        body: Box<Statements>,
    ) -> Self {
        Self {
            init_decl,
            continue_expression,
            modify_expression,
            body,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

impl_locatable!(
    BreakStatement,
    ContinueStatement,
    ForStatement,
    FunctionStatement,
    IfStatement,
    LetStatement,
    ReturnStatement,
    Statement,
    Statements,
    StructStatement,
    WhileStatement
);
