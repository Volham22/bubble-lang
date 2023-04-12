use super::{
    expressions::Expression,
    location::{Locatable, TokenLocation},
    types::Type,
    TypeKind,
};

#[derive(Debug)]
pub enum GlobalStatement {
    Function(FunctionStatement),
    Struct(StructStatement),
}

pub type FunctionParameter = (TypeKind, String);

#[derive(Debug)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeKind,
    pub body: Statements,
    location: TokenLocation,
}

impl FunctionStatement {
    pub fn new(
        tk_begin: usize,
        tk_end: usize,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: TypeKind,
        body: Statements,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

impl Locatable for FunctionStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct StructStatement {
    pub name: String,
    pub fields: Vec<FunctionParameter>,
    location: TokenLocation,
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
        }
    }
}

impl Locatable for StructStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
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
        self.statements.push(stmt);
    }
}

impl Locatable for Statements {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
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

impl Locatable for Statement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub enum StatementKind {
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Expression(Box<Expression>),
}

#[derive(Debug)]
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

impl Locatable for IfStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
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

impl Locatable for WhileStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct ForStatement {
    pub init_identifier: String,
    pub init_expression: Box<Expression>,
    pub init_type: Option<Type>,
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
        init_identifier: String,
        init_expression: Box<Expression>,
        init_type: Option<Type>,
        continue_expression: Box<Expression>,
        modify_expression: Box<Expression>,
        body: Box<Statements>,
    ) -> Self {
        Self {
            init_identifier,
            init_expression,
            init_type,
            continue_expression,
            modify_expression,
            body,
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }
}

impl Locatable for ForStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}
