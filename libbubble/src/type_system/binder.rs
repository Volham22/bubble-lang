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
pub enum BinderError {
    #[error("undeclared variable {name:?}")]
    UndeclaredVariable {
        location: TokenLocation,
        name: String,
    },
    #[error("undeclared struct {name:?}")]
    UndeclaredStruct {
        location: TokenLocation,
        name: String,
    },
    #[error("undeclared function {name:?}")]
    UndeclaredFunction {
        location: TokenLocation,
        name: String,
    },
    #[error("'return' outside a function")]
    BadReturn { location: TokenLocation },
    #[error("'break' outside a loop")]
    BadBreak { location: TokenLocation },
    #[error("'continue' outside a loop")]
    BadContinue { location: TokenLocation },
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
    pub fn bind_statements(&mut self, stmts: &mut [GlobalStatement]) -> Result<(), BinderError> {
        for stmt in stmts.iter_mut() {
            stmt.accept_mut(self)?;
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

impl MutableVisitor<BinderError> for Binder {
    fn visit_function(&mut self, stmt: &mut FunctionStatement) -> Result<(), BinderError> {
        self.functions_statements
            .insert(stmt.name.to_string(), stmt.clone());

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

        self.in_function = true;
        self.visit_statements(&mut stmt.body)?;
        self.in_function = false;
        self.local_variables.delete_scope();

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &mut StructStatement) -> Result<(), BinderError> {
        self.struct_statement
            .insert(stmt.name.to_string(), stmt.clone());

        for (kind, _) in &mut stmt.fields {
            kind.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &mut LetStatement) -> Result<(), BinderError> {
        self.local_variables.insert_symbol(&stmt.name, stmt.clone());
        stmt.init_exp.accept_mut(self)?;
        Ok(())
    }

    fn visit_if(&mut self, stmt: &mut IfStatement) -> Result<(), BinderError> {
        stmt.condition.accept_mut(self)?;

        self.local_variables.new_scope();
        for then_stmt in &mut stmt.then_clause.statements {
            then_stmt.kind.accept_mut(self)?;
        }
        self.local_variables.delete_scope();

        if let Some(else_clause) = &mut stmt.else_clause {
            self.local_variables.new_scope();
            for else_stmt in &mut else_clause.statements {
                else_stmt.kind.accept_mut(self)?;
            }
            self.local_variables.delete_scope();
        }
        Ok(())
    }

    fn visit_while(&mut self, stmt: &mut WhileStatement) -> Result<(), BinderError> {
        stmt.condition.accept_mut(self)?;

        self.begin_loop();
        for while_stmt in &mut stmt.body.statements {
            while_stmt.kind.accept_mut(self)?;
        }
        self.end_loop();

        Ok(())
    }

    fn visit_for(&mut self, stmt: &mut ForStatement) -> Result<(), BinderError> {
        stmt.init_expression.accept_mut(self)?;

        if let Some(ty) = &mut stmt.init_type {
            ty.kind.accept_mut(self)?;
        }

        self.begin_loop();
        self.local_variables.insert_symbol(
            &stmt.init_identifier,
            LetStatement::new(
                stmt.get_location().begin,
                stmt.get_location().end,
                stmt.init_identifier.clone(),
                stmt.init_type.clone().map(|t| t.kind),
                stmt.init_expression.clone(),
            ),
        );

        stmt.modify_expression.accept_mut(self)?;
        stmt.continue_expression.accept_mut(self)?;

        for for_stmt in &mut stmt.body.statements {
            for_stmt.kind.accept_mut(self)?;
        }
        self.end_loop();

        Ok(())
    }

    fn visit_return(&mut self, stmt: &mut ReturnStatement) -> Result<(), BinderError> {
        if !self.in_function {
            Err(BinderError::BadReturn {
                location: stmt.get_location().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_break(&mut self, stmt: &mut BreakStatement) -> Result<(), BinderError> {
        if self.nested_loop == 0 {
            Err(BinderError::BadBreak {
                location: stmt.get_location().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_continue(&mut self, stmt: &mut ContinueStatement) -> Result<(), BinderError> {
        if self.nested_loop == 0 {
            Err(BinderError::BadContinue {
                location: stmt.get_location().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_literal(&mut self, expr: &mut Literal) -> Result<(), BinderError> {
        if let LiteralType::Identifier(name) = &expr.literal_type {
            if self.local_variables.find_symbol(name).is_none() {
                Err(BinderError::UndeclaredVariable {
                    location: expr.get_location().clone(),
                    name: name.clone(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn visit_call(&mut self, expr: &mut Call) -> Result<(), BinderError> {
        let declaration = self.functions_statements.get(&expr.callee);
        if declaration.is_none() {
            return Err(BinderError::UndeclaredFunction {
                location: expr.get_location().clone(),
                name: expr.callee.to_string(),
            });
        }

        expr.set_definition(Definition::Function(declaration.unwrap().clone()));

        for arg in &mut expr.arguments {
            arg.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, expr: &mut Type) -> Result<(), BinderError> {
        match &expr.kind {
            TypeKind::Identifier(name) => {
                let declaration = self.struct_statement.get(name);
                if let Some(dec) = declaration {
                    expr.set_definition(Definition::Struct(dec.clone()));
                    Ok(())
                } else {
                    Err(BinderError::UndeclaredStruct {
                        location: TokenLocation::new(0, 0), // FIXME: Add proper location for type identifier
                        name: name.clone(),
                    })
                }
            }
            _ => Ok(()),
        }
    }
}
