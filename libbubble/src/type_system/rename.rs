use std::convert::Infallible;

use crate::ast::{
    ForStatement, FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal,
    MutableVisitor, StructStatement, WhileStatement,
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
            self.visit_global_statement(stmt)?;
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
impl<'ast> MutableVisitor<'ast, Infallible> for Renamer {
    fn visit_function(&mut self, stmt: &mut FunctionStatement) -> Result<(), Infallible> {
        self.variables.new_scope();

        for param_stmt in stmt.parameters.iter_mut() {
            param_stmt.name = self.new_symbol(&param_stmt.name); // rename function parameter

            self.variables
                .insert_symbol(&param_stmt.name, param_stmt.clone());
        }

        if let Some(body) = stmt.body.as_mut() {
            self.visit_statements(body)?;
        }

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
        self.visit_expression(stmt.init_exp.as_mut().expect("Let has no init statement"))?;
        self.variables.insert_symbol(&prev_name, stmt.clone());

        Ok(())
    }

    fn visit_if(&mut self, stmt: &mut IfStatement) -> Result<(), Infallible> {
        self.visit_expression(&mut stmt.condition)?;
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
        self.visit_expression(&mut stmt.condition)?;

        self.variables.new_scope();
        self.visit_statements(&mut stmt.body)?;
        self.variables.delete_scope();

        Ok(())
    }

    fn visit_for(&mut self, stmt: &mut ForStatement) -> Result<(), Infallible> {
        self.variables.new_scope();
        self.visit_let(&mut stmt.init_decl)?;
        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_expression(&mut stmt.continue_expression)?;
        self.visit_statements(&mut stmt.body)?;
        self.variables.delete_scope();

        Ok(())
    }

    fn visit_literal(&mut self, _: &mut Literal) -> Result<(), Infallible> {
        // if let LiteralType::Identifier(ref id) = literal.literal_type {
        //     match self.variables.find_symbol(id) {
        //         Some(decl) => {
        //             literal.set_definition(Definition::LocalVariable(decl.clone()));
        //             literal.literal_type = LiteralType::Identifier(decl.name.clone());
        //         }
        //         None => unreachable!("Undeclared symbol in renamer!"),
        //     }
        // }

        Ok(())
    }
}
