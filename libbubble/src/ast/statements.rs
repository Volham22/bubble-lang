use super::{
    expressions::Expression,
    location::{Locatable, TokenLocation},
    types::Type,
    visitor::Visitor,
    TypeKind,
};

#[derive(Debug)]
pub enum GlobalStatement {
    Function(FunctionStatement),
    Struct(StructStatement),
    Let(LetStatement),
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

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_function(self);
    }
}

impl Locatable for FunctionStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct LetStatement {
    pub name: String,
    pub declaration_type: Option<TypeKind>,
    pub init_exp: Box<Expression>,
    location: TokenLocation,
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
        }
    }

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_let(self);
    }
}

impl Locatable for LetStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub exp: Box<Expression>,
    location: TokenLocation,
}

impl ReturnStatement {
    pub fn new(begin_tk: usize, end_tk: usize, exp: Box<Expression>) -> Self {
        Self {
            exp,
            location: TokenLocation::new(begin_tk, end_tk),
        }
    }

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_return(self);
    }
}

impl Locatable for ReturnStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct BreakStatement {
    location: TokenLocation,
}

impl BreakStatement {
    pub fn new(tk_begin: usize, tk_end: usize) -> Self {
        Self {
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_break(self);
    }
}

impl Locatable for BreakStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}

#[derive(Debug)]
pub struct ContinueStatement {
    location: TokenLocation,
}

impl ContinueStatement {
    pub fn new(tk_begin: usize, tk_end: usize) -> Self {
        Self {
            location: TokenLocation::new(tk_begin, tk_end),
        }
    }

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_continue(self);
    }
}

impl Locatable for ContinueStatement {
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

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_struct(self);
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
    Return(ReturnStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Expression(Box<Expression>),
}
impl StatementKind {
    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        match self {
            StatementKind::If(stmt) => stmt.accept(v),
            StatementKind::While(stmt) => stmt.accept(v),
            StatementKind::For(stmt) => stmt.accept(v),
            StatementKind::Return(stmt) => stmt.accept(v),
            StatementKind::Break(stmt) => stmt.accept(v),
            StatementKind::Continue(stmt) => stmt.accept(v),
            StatementKind::Expression(stmt) => stmt.accept(v),
        }
    }
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

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_if(self);
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

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_while(self);
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

    pub fn accept<T: Visitor + ?Sized>(&self, v: &T) {
        v.visit_for(self);
    }
}

impl Locatable for ForStatement {
    fn get_location(&self) -> &TokenLocation {
        &self.location
    }
}
