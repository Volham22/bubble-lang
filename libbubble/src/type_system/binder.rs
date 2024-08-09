use std::collections::HashMap;

use crate::{
    ast::{
        Bindable, BreakStatement, Call, ContinueStatement, Definition, Expression, ForStatement,
        FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, LiteralType,
        Locatable, MutableVisitor, ReturnStatement, StructAccess, StructStatement, Type, TypeKind,
        WhileStatement,
    },
    desugar,
};

use super::{errors::BinderError, utils::ScopedMap};

#[derive(Default)]
pub struct Binder<'a> {
    module_name: &'a str,
    functions_statements: HashMap<String, *const FunctionStatement>,
    struct_statement: HashMap<String, *const StructStatement>,
    local_variables: ScopedMap<*const LetStatement>,
    nested_loop: usize,
    in_function: bool,
}

impl<'m> Binder<'m> {
    pub fn new(module_name: &'m str) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }

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

impl<'ast> MutableVisitor<'ast, BinderError> for Binder<'_> {
    fn visit_function(&mut self, stmt: &'ast mut FunctionStatement) -> Result<(), BinderError> {
        for param in stmt.parameters.iter_mut() {
            self.visit_let(param)?;
        }

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

        self.visit_type(&mut stmt.return_type)?;

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), BinderError> {
        self.struct_statement.insert(
            desugar::SymbolMangler::mangle_qualified(self.module_name, &stmt.name),
            stmt,
        );

        for (ty, _) in stmt.fields.iter_mut() {
            self.visit_type(ty)?;
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), BinderError> {
        self.local_variables.insert_symbol(&stmt.name, stmt);

        // Bind type identifier to its concrete type
        if let Some(ty) = &mut stmt.declaration_type {
            self.visit_type(ty)?;
        }

        // A parameter is a let statement in the ast but as no init expression.
        // Init expression are enforced in the grammar and are always present otherwise.
        if let Some(init_exp) = stmt.init_exp.as_mut() {
            self.visit_expression(init_exp)?;
        }

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
        let mangle_name = desugar::SymbolMangler::mangle_qualified(self.module_name, &expr.callee);
        let declaration = match self.functions_statements.get(&mangle_name) {
            Some(d) => Some(d),
            // Try without name mangling
            None => self.functions_statements.get(&expr.callee),
        };
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
        match &mut expr.kind {
            TypeKind::Identifier(name) => {
                let name = name.clone();
                let declaration = self.struct_statement.get(&name);
                if let Some(dec) = declaration {
                    expr.set_definition(Definition::Struct(*dec));
                    Ok(())
                } else if let Some(dec) =
                    self.struct_statement
                        .get(&desugar::SymbolMangler::mangle_qualified(
                            self.module_name,
                            &name,
                        ))
                {
                    expr.set_definition(Definition::Struct(*dec));
                    Ok(())
                } else {
                    Err(BinderError::UndeclaredStruct {
                        location: expr.get_location().clone(),
                        name,
                    })
                }
            }
            TypeKind::Ptr(ty) => self.visit_type(ty),
            TypeKind::Array { array_type, .. } => self.visit_type(array_type),
            _ => Ok(()),
        }
    }

    fn visit_struct_access(&mut self, stmt: &'ast mut StructAccess) -> Result<(), BinderError> {
        self.visit_expression(&mut stmt.identifier)?;
        if let Expression::StructAccess(sa) = stmt.identifier.as_ref() {
            stmt.set_definition(*sa.get_definition());
            return Ok(());
        }

        let struct_name = match &stmt.identifier.as_ref() {
            Expression::Literal(Literal {
                literal_type: LiteralType::Identifier(struct_name),
                ..
            }) => struct_name,
            _ => {
                return Ok(());
            }
        };

        let strct = self.local_variables.find_symbol(struct_name).ok_or(
            BinderError::UndeclaredVariable {
                location: stmt.get_location().to_owned(),
                name: struct_name.to_owned(),
            },
        )?;

        stmt.definition = Some(Definition::LocalVariable(*strct));
        Ok(())
    }
}
