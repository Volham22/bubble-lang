use std::convert::Infallible;

use crate::ast::{
    Bindable, Definition, Expression, ForStatement, FunctionStatement, GlobalStatement,
    IfStatement, LetStatement, Literal, LiteralType, Locatable, MutableVisitor, StructStatement,
    WhileStatement,
};

use super::utils::ScopedMap;

#[derive(Default)]
pub struct Renamer {
    symbol_count: u32,
    variables: ScopedMap<LetStatement>,
}

impl Renamer {
    pub fn rename_statements(&mut self, stmts: &mut [GlobalStatement]) -> Result<(), Infallible> {
        for stmt in stmts {
            stmt.accept_mut(self)?;
        }

        Ok(())
    }
    fn new_symbol(&mut self, symbol: &str) -> String {
        self.symbol_count += 1;
        format!("{}_{}", symbol, self.symbol_count)
    }
}

// Renamer should never fail because errors must be caught by the previous
// compiler passes. Infallible should be replaced by `!` in future Rust's versions
impl MutableVisitor<Infallible> for Renamer {
    fn visit_function(&mut self, stmt: &mut FunctionStatement) -> Result<(), Infallible> {
        let location = stmt.get_location().clone();
        self.variables.new_scope();

        // TODO: Function parameters could be let statement.
        for (kind, parameter_name) in stmt.parameters.iter_mut() {
            *parameter_name = self.new_symbol(parameter_name); // rename function parameter

            self.variables.insert_symbol(
                parameter_name,
                LetStatement::new(
                    location.begin,
                    location.end,
                    parameter_name.to_string(),
                    Some(kind.clone()),
                    Box::new(Expression::Literal(Literal::new(
                        location.begin,
                        location.end,
                        crate::ast::LiteralType::True,
                    ))),
                ),
            );
        }

        self.visit_statements(&mut stmt.body)?;
        self.variables.delete_scope();

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &mut StructStatement) -> Result<(), Infallible> {
        stmt.name = self.new_symbol(&stmt.name);
        Ok(())
    }

    fn visit_let(&mut self, stmt: &mut LetStatement) -> Result<(), Infallible> {
        let prev_name = stmt.name.clone();
        stmt.name = self.new_symbol(&stmt.name);
        stmt.init_exp.accept_mut(self)?;
        self.variables.insert_symbol(&prev_name, stmt.clone());

        Ok(())
    }

    fn visit_if(&mut self, stmt: &mut IfStatement) -> Result<(), Infallible> {
        stmt.condition.accept_mut(self)?;
        self.variables.new_scope();
        self.visit_statements(&mut stmt.then_clause)?;
        self.variables.delete_scope();

        if let Some(ref mut stmts) = stmt.else_clause {
            self.variables.new_scope();
            self.visit_statements(stmts)?;
            self.variables.delete_scope();
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &mut WhileStatement) -> Result<(), Infallible> {
        stmt.condition.accept_mut(self)?;

        self.variables.new_scope();
        self.visit_statements(&mut stmt.body)?;
        self.variables.delete_scope();

        Ok(())
    }

    fn visit_for(&mut self, stmt: &mut ForStatement) -> Result<(), Infallible> {
        self.variables.new_scope();
        stmt.init_decl.accept_mut(self)?;
        stmt.modify_expression.accept_mut(self)?;
        stmt.continue_expression.accept_mut(self)?;
        self.visit_statements(&mut stmt.body)?;
        self.variables.delete_scope();

        Ok(())
    }

    fn visit_literal(&mut self, literal: &mut Literal) -> Result<(), Infallible> {
        if let LiteralType::Identifier(ref id) = literal.literal_type {
            match self.variables.find_symbol(id) {
                Some(decl) => {
                    literal.set_definition(Definition::LocalVariable(decl.clone()));
                    literal.literal_type = LiteralType::Identifier(decl.name.clone());
                }
                None => unreachable!("Undeclared symbol in renamer!"),
            }
        }

        Ok(())
    }
}
