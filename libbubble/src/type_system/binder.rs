use std::collections::HashMap;

use crate::ast::{
    Bindable, BreakStatement, Call, ContinueStatement, Definition, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, LiteralType, Locatable,
    MutableVisitor, ReturnStatement, StructStatement, TokenLocation, Type, TypeKind,
    WhileStatement,
};
use thiserror::Error;

use super::utils::ScopedMap;

#[derive(Error, Debug)]
pub enum BinderError<'ast> {
    #[error("undeclared variable {name:?}")]
    UndeclaredVariable {
        location: &'ast TokenLocation,
        name: String,
    },
    #[error("undeclared struct {name:?}")]
    UndeclaredStruct {
        location: &'ast TokenLocation,
        name: String,
    },
    #[error("undeclared function {name:?}")]
    UndeclaredFunction {
        location: &'ast TokenLocation,
        name: String,
    },
    #[error("'return' outside a function")]
    BadReturn { location: &'ast TokenLocation },
    #[error("'break' outside a loop")]
    BadBreak { location: &'ast TokenLocation },
    #[error("'continue' outside a loop")]
    BadContinue { location: &'ast TokenLocation },
}

#[derive(Default)]
pub struct Binder {
    functions_statements: HashMap<String, FunctionStatement>,
    struct_statement: HashMap<String, StructStatement>,
    local_variables: ScopedMap<LetStatement>,
    nested_loop: usize,
    in_function: bool,
}

impl Binder {
    pub fn bind_statements<'ast>(
        &mut self,
        stmts: &'ast mut [GlobalStatement],
    ) -> Result<(), BinderError<'ast>> {
        for stmt in stmts {
            self.visit_global_statement(stmt)?;
        }

        Ok(())
    }

    fn begin_loop(&mut self) {
        self.nested_loop += 1;
        self.local_variables.new_scope();
    }

    fn end_loop(&mut self) {
        self.nested_loop -= 1;
        self.local_variables.delete_scope();
    }
}

impl<'ast> MutableVisitor<'ast, BinderError<'ast>> for Binder {
    fn visit_function(
        &mut self,
        stmt: &'ast mut FunctionStatement,
    ) -> Result<(), BinderError<'ast>> {
        self.local_variables.new_scope();
        let location = stmt.get_location();

        // We treat functions parameters as simple declarations as it'll simplify the rest of our
        // implementation.
        // TODO: Investigate if it's possible to do it directly in the ast
        for (kind, parameter_name) in &stmt.parameters {
            self.local_variables.insert_symbol(
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

        self.functions_statements
            .insert(stmt.name.to_string(), stmt.clone());
        self.in_function = true;
        self.visit_statements(&mut stmt.body)?;
        self.in_function = false;
        self.local_variables.delete_scope();

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), BinderError<'ast>> {
        self.struct_statement
            .insert(stmt.name.to_string(), stmt.clone());

        for (kind, _) in &mut stmt.fields {
            self.visit_type_kind(kind)?;
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), BinderError<'ast>> {
        self.local_variables.insert_symbol(&stmt.name, stmt.clone());
        self.visit_expression(&mut stmt.init_exp)?;
        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast mut IfStatement) -> Result<(), BinderError<'ast>> {
        self.visit_expression(&mut stmt.condition)?;

        self.local_variables.new_scope();
        self.visit_statements_vec(&mut stmt.then_clause.statements)?;
        self.local_variables.delete_scope();

        if let Some(else_clause) = &mut stmt.else_clause {
            self.local_variables.new_scope();
            self.visit_statements_vec(&mut else_clause.statements)?;
            self.local_variables.delete_scope();
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast mut WhileStatement) -> Result<(), BinderError<'ast>> {
        self.visit_expression(&mut stmt.condition)?;

        self.begin_loop();
        self.visit_statements_vec(&mut stmt.body.statements)?;
        self.end_loop();

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast mut ForStatement) -> Result<(), BinderError<'ast>> {
        self.begin_loop();

        self.visit_let(&mut stmt.init_decl)?;
        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_expression(&mut stmt.continue_expression)?;
        self.visit_statements_vec(&mut stmt.body.statements)?;

        self.end_loop();

        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), BinderError<'ast>> {
        if !self.in_function {
            Err(BinderError::BadReturn {
                location: stmt.get_location(),
            })
        } else {
            self.visit_expression(&mut stmt.exp)?;
            Ok(())
        }
    }

    fn visit_break(&mut self, stmt: &'ast mut BreakStatement) -> Result<(), BinderError<'ast>> {
        if self.nested_loop == 0 {
            Err(BinderError::BadBreak {
                location: stmt.get_location(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_continue(
        &mut self,
        stmt: &'ast mut ContinueStatement,
    ) -> Result<(), BinderError<'ast>> {
        if self.nested_loop == 0 {
            Err(BinderError::BadContinue {
                location: stmt.get_location(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), BinderError<'ast>> {
        if let LiteralType::Identifier(name) = &expr.literal_type {
            match self.local_variables.find_symbol(name) {
                Some(var) => {
                    expr.set_definition(Definition::LocalVariable(var.clone()));
                    Ok(())
                }
                None => Err(BinderError::UndeclaredVariable {
                    location: expr.get_location(),
                    name: name.clone(),
                }),
            }
        } else {
            Ok(())
        }
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), BinderError<'ast>> {
        let declaration = self.functions_statements.get(&expr.callee);
        if declaration.is_none() {
            return Err(BinderError::UndeclaredFunction {
                location: expr.get_location(),
                name: expr.callee.to_string(),
            });
        }

        let dec = declaration.unwrap();
        expr.set_definition(Definition::Function((*dec).clone()));

        for arg in &mut expr.arguments {
            self.visit_expression(arg)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, expr: &'ast mut Type) -> Result<(), BinderError<'ast>> {
        match &expr.kind {
            TypeKind::Identifier(name) => {
                let declaration = self.struct_statement.get(name);
                if let Some(dec) = declaration {
                    expr.set_definition(Definition::Struct(dec.clone()));
                    Ok(())
                } else {
                    Err(BinderError::UndeclaredStruct {
                        location: expr.get_location(),
                        name: name.clone(),
                    })
                }
            }
            _ => Ok(()),
        }
    }
}
