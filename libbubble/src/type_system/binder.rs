use std::collections::HashMap;

use crate::ast::{
    Bindable, BreakStatement, Call, ContinueStatement, Definition, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, LiteralType, Locatable,
    MutableVisitor, ReturnStatement, StructStatement, Type, TypeKind, WhileStatement,
};

use super::{errors::BinderError, utils::ScopedMap};

#[derive(Default)]
pub struct Binder {
    functions_statements: HashMap<String, *const FunctionStatement>,
    struct_statement: HashMap<String, *const StructStatement>,
    local_variables: ScopedMap<*const LetStatement>,
    nested_loop: usize,
    in_function: bool,
}

impl Binder {
    pub fn bind_statements(&mut self, stmts: &mut [GlobalStatement]) -> Result<(), BinderError> {
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

    fn is_subscriptable(expr: &Expression) -> bool {
        match expr {
            Expression::Literal(lit) => {
                matches!(
                    lit.literal_type,
                    LiteralType::Identifier(_) | LiteralType::String(_)
                )
            }
            Expression::Call(_) => true,
            _ => false,
        }
    }
}

impl<'ast> MutableVisitor<'ast, BinderError> for Binder {
    fn visit_function(&mut self, stmt: &'ast mut FunctionStatement) -> Result<(), BinderError> {
        self.functions_statements
            .insert(stmt.name.to_string(), stmt);

        if !stmt.is_extern {
            self.local_variables.new_scope();
            // We treat functions parameters as simple declarations as it'll simplify the rest of our
            // implementation.
            // TODO: Investigate if it's possible to do it directly in the ast
            for let_stmt in &stmt.parameters {
                self.local_variables.insert_symbol(&let_stmt.name, let_stmt);
            }

            self.in_function = true;
            self.visit_statements(stmt.body.as_mut().unwrap())?;
            self.in_function = false;
            self.local_variables.delete_scope();
        }

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), BinderError> {
        self.struct_statement.insert(stmt.name.to_string(), stmt);

        for (kind, _) in &mut stmt.fields {
            self.visit_type_kind(kind)?;
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), BinderError> {
        self.local_variables.insert_symbol(&stmt.name, stmt);
        self.visit_expression(
            stmt.init_exp
                .as_mut()
                .expect("variable declaration has no init type!"),
        )?;

        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast mut IfStatement) -> Result<(), BinderError> {
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

    fn visit_while(&mut self, stmt: &'ast mut WhileStatement) -> Result<(), BinderError> {
        self.visit_expression(&mut stmt.condition)?;

        self.begin_loop();
        self.visit_statements_vec(&mut stmt.body.statements)?;
        self.end_loop();

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast mut ForStatement) -> Result<(), BinderError> {
        self.begin_loop();

        self.visit_let(&mut stmt.init_decl)?;
        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_expression(&mut stmt.continue_expression)?;
        self.visit_statements_vec(&mut stmt.body.statements)?;

        self.end_loop();

        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), BinderError> {
        if !self.in_function {
            Err(BinderError::BadReturn {
                location: stmt.get_location().clone(),
            })
        } else {
            if let Some(ref mut exp) = stmt.exp {
                self.visit_expression(exp)?;
            }

            Ok(())
        }
    }

    fn visit_break(&mut self, stmt: &'ast mut BreakStatement) -> Result<(), BinderError> {
        if self.nested_loop == 0 {
            Err(BinderError::BadBreak {
                location: stmt.get_location().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_continue(&mut self, stmt: &'ast mut ContinueStatement) -> Result<(), BinderError> {
        if self.nested_loop == 0 {
            Err(BinderError::BadContinue {
                location: stmt.get_location().clone(),
            })
        } else {
            Ok(())
        }
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), BinderError> {
        match &expr.literal_type {
            LiteralType::Identifier(name) => match self.local_variables.find_symbol(name) {
                Some(var) => expr.set_definition(Definition::LocalVariable(*var)),
                None => {
                    return Err(BinderError::UndeclaredVariable {
                        location: expr.get_location().clone(),
                        name: name.clone(),
                    })
                }
            },
            LiteralType::ArrayAccess(array_access)
                if Self::is_subscriptable(&array_access.identifier) =>
            {
                let name = match array_access.identifier.as_ref() {
                    Expression::Literal(l) => match &l.literal_type {
                        LiteralType::Identifier(name) => name,
                        _ => unreachable!(),
                    },
                    Expression::Call(c) => &c.callee,
                    _ => unreachable!(),
                };

                match self.local_variables.find_symbol(name) {
                    Some(var) => expr.set_definition(Definition::LocalVariable(*var)),
                    None => match self.functions_statements.get(name) {
                        Some(f) => expr.set_definition(Definition::Function(*f)),
                        None => {
                            return Err(BinderError::UndeclaredVariable {
                                location: expr.get_location().clone(),
                                name: name.clone(),
                            });
                        }
                    },
                }
            }
            LiteralType::ArrayAccess(_) => {
                return Err(BinderError::NotSubscriptable {
                    location: expr.get_location().clone(),
                });
            }
            _ => (),
        };

        // Bind the array access identifier too
        if let LiteralType::ArrayAccess(aa) = &mut expr.literal_type {
            self.visit_expression(&mut aa.identifier)?;
            self.visit_expression(&mut aa.index)?;
        }

        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), BinderError> {
        let declaration = self.functions_statements.get(&expr.callee);
        if declaration.is_none() {
            return Err(BinderError::UndeclaredFunction {
                location: expr.get_location().clone(),
                name: expr.callee.to_string(),
            });
        }

        let dec = *declaration.unwrap();
        expr.set_definition(Definition::Function(dec));

        for arg in &mut expr.arguments {
            self.visit_expression(arg)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, expr: &'ast mut Type) -> Result<(), BinderError> {
        match &expr.kind {
            TypeKind::Identifier(name) => {
                let declaration = self.struct_statement.get(name);
                if let Some(dec) = declaration {
                    expr.set_definition(Definition::Struct(*dec));
                    Ok(())
                } else {
                    Err(BinderError::UndeclaredStruct {
                        location: expr.get_location().clone(),
                        name: name.clone(),
                    })
                }
            }
            _ => Ok(()),
        }
    }
}
