use crate::ast::*;

use super::{type_checker::TypeCheckerError, Typable, Type};

#[derive(Debug, Default)]
pub(crate) struct IntegerInference {
    current_function: Option<Type>,
    is_int: bool,
}

impl IntegerInference {
    pub fn infer_statements(
        &mut self,
        stmts: &mut [GlobalStatement],
    ) -> Result<(), TypeCheckerError> {
        for stmt in stmts {
            self.visit_global_statement(stmt)?;
        }

        Ok(())
    }
}

/// This visitor is here to infer proper integer types to literal expressions
/// such as `42`. This visitor only takes care of integer like types and does
/// not set any other types.
///
/// This visitor is executed before the actual type checker and try to guess
/// wheter an int literal should be an signed integer or not and its size.
///
/// This type checker can fail if an integer type can't be directly infered.
/// The following code will produce an error:
///
/// ```bubble
/// let a = 2;
/// ```
impl<'ast> MutableVisitor<'ast, TypeCheckerError> for IntegerInference {
    fn visit_global_statement(
        &mut self,
        stmt: &'ast mut GlobalStatement,
    ) -> Result<(), TypeCheckerError> {
        match stmt {
            GlobalStatement::Function(f) => {
                self.current_function = Some(f.ty.as_ref().unwrap().clone());
                self.visit_function(f)?;
                self.current_function = None;

                Ok(())
            }
            GlobalStatement::Struct(s) => self.visit_struct(s),
            GlobalStatement::Let(l) => self.visit_let(l),
        }
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), TypeCheckerError> {
        self.visit_expression(&mut stmt.init_exp)?;

        if self.is_int {
            let statement_ty = stmt.get_type().clone();

            match stmt.declaration_type {
                Some(_) => match stmt.init_exp.as_mut() {
                    Expression::Literal(l) => {
                        l.set_type(statement_ty);
                        Ok(())
                    }
                    _ => unreachable!(),
                },
                None => Err(TypeCheckerError::InferenceError(
                    stmt.get_location().clone(),
                )),
            }
        } else {
            Ok(())
        }
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), TypeCheckerError> {
        // Do nothing it the return type is `void`. Incompatible return types errors
        // are caught by the actual type checker.
        if stmt.exp.is_none() {
            return Ok(());
        }

        println!("{:?}", stmt.exp);
        self.visit_expression(stmt.exp.as_mut().unwrap())?;

        if self.is_int {
            let current_fn = self.current_function.as_ref().unwrap();

            match current_fn {
                Type::Function { return_type, .. } => match stmt.exp.as_mut().unwrap().as_mut() {
                    Expression::Literal(l) => l.set_type(return_type.as_ref().clone()),
                    _ => unreachable!(),
                },
                _ => unreachable!("function has no function type!"),
            }

            self.is_int = false;
        }

        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), TypeCheckerError> {
        match expr.literal_type {
            LiteralType::Integer(_) => {
                if let Type::Int = expr.get_type() {
                    self.is_int = true;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
