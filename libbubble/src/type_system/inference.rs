use std::convert::Infallible;

use crate::ast::*;

use super::{errors::TypeCheckerError, Typable, Type};

pub(crate) struct ExpressionTypeSetter<'ty> {
    new_type: &'ty Type,
}

impl<'ty> ExpressionTypeSetter<'ty> {
    pub fn new(new_type: &'ty Type) -> Self {
        Self { new_type }
    }

    pub fn set_type_recusively(&mut self, expr: &mut Expression) {
        self.visit_expression(expr).expect("Should never fail");
    }
}

impl<'ast, 'ty> MutableVisitor<'ast, Infallible> for ExpressionTypeSetter<'ty> {
    fn visit_binary_operation(
        &mut self,
        expr: &'ast mut BinaryOperation,
    ) -> Result<(), Infallible> {
        self.visit_expression(&mut expr.left)?;
        if let Some(right) = &mut expr.right {
            self.visit_expression(right)?;
        }

        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), Infallible> {
        match &mut expr.literal_type {
            LiteralType::ArrayAccess(aa) => {
                aa.set_type(self.new_type.clone());
            }
            LiteralType::Null(n) => n.set_type(self.new_type.clone()),
            _ => (),
        }

        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), Infallible> {
        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), Infallible> {
        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_array_initializer(
        &mut self,
        expr: &'ast mut ArrayInitializer,
    ) -> Result<(), Infallible> {
        for exp in expr.values.iter_mut() {
            self.visit_expression(exp)?;
        }

        expr.set_type(Type::Array {
            size: expr.values.len() as u32,
            array_type: Box::new(self.new_type.clone()),
        });

        Ok(())
    }
}

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
        self.visit_expression(
            stmt.init_exp
                .as_mut()
                .expect("Let statement has no init exp"),
        )?;

        if self.is_int {
            match stmt.declaration_type {
                Some(_) => {
                    let statement_ty = match stmt.get_type() {
                        // If it's an array we need to set inner expression type to the base type
                        Type::Array { array_type, .. } => array_type.as_ref().clone(),
                        _ => stmt.get_type().clone(),
                    };
                    let mut setter = ExpressionTypeSetter::new(&statement_ty);
                    setter.set_type_recusively(
                        stmt.init_exp
                            .as_mut()
                            .expect("Let statement has no init exp"),
                    );

                    Ok(())
                }
                None => Err(TypeCheckerError::InferenceError(
                    stmt.get_location().clone(),
                )),
            }
        } else {
            Ok(())
        }
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), TypeCheckerError> {
        if expr.get_definition().is_function() {
            for i in 0..expr.arguments.len() {
                self.visit_expression(expr.arguments.get_mut(i).expect("unreachable"))?;

                // Infer only Int type
                if !self.is_int {
                    continue;
                }

                let expected_type = expr
                    .get_function_def()
                    .parameters
                    .get(i)
                    .unwrap()
                    .ty
                    .clone()
                    .unwrap();

                if !expected_type.is_compatible_with(&Type::Int) {
                    return Err(TypeCheckerError::BadParameter {
                        name: expr.get_function_def().name.clone(),
                        expected_type,
                        got: Type::Int,
                    });
                }

                let mut setter = ExpressionTypeSetter::new(&expected_type);
                setter.set_type_recusively(expr.arguments.get_mut(i).expect("unreachable"));
                self.is_int = false;
            }

            Ok(())
        } else {
            Err(TypeCheckerError::NotCallable(expr.get_definition().clone()))
        }
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), TypeCheckerError> {
        // Do nothing it the return type is `void`. Incompatible return types errors
        // are caught by the actual type checker.
        if stmt.exp.is_none() {
            return Ok(());
        }

        self.visit_expression(stmt.exp.as_mut().unwrap())?;

        if self.is_int {
            let current_fn = self.current_function.as_ref().unwrap();

            match current_fn {
                Type::Function { return_type, .. } => {
                    let mut setter = ExpressionTypeSetter::new(return_type);
                    setter.set_type_recusively(stmt.exp.as_mut().unwrap().as_mut());
                }
                _ => unreachable!("function has no function type!"),
            }

            self.is_int = false;
        }

        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), TypeCheckerError> {
        match expr.literal_type {
            LiteralType::Integer(_) | LiteralType::ArrayAccess(_) => {
                if let Type::Int = expr.get_type() {
                    self.is_int = true;
                }
            }
            _ => {
                self.is_int = false;
            }
        };

        // If the index has a type Int. set it to I64.
        if let LiteralType::ArrayAccess(ArrayAccess { index, .. }) = &mut expr.literal_type {
            if let Type::Int = index.get_type() {
                let mut setter = ExpressionTypeSetter::new(&Type::I64);
                setter.set_type_recusively(index);
            }
        }

        Ok(())
    }

    fn visit_binary_operation(
        &mut self,
        expr: &'ast mut BinaryOperation,
    ) -> Result<(), TypeCheckerError> {
        if expr.right.is_none() {
            self.visit_expression(&mut expr.left)
        } else {
            self.visit_expression(&mut expr.left)?;
            let is_int_left = self.is_int;
            self.visit_expression(expr.right.as_mut().unwrap())?;
            let is_int_right = self.is_int;

            match (is_int_left, is_int_right) {
                (true, true) => {
                    let mut setter = ExpressionTypeSetter::new(&Type::I64);
                    setter.set_type_recusively(&mut expr.left);
                    setter.set_type_recusively(expr.right.as_mut().unwrap());

                    Ok(())
                }
                (true, false) => {
                    let ty = expr.right.as_ref().unwrap().get_type();
                    let mut setter = ExpressionTypeSetter::new(ty);
                    setter.set_type_recusively(&mut expr.left);
                    Ok(())
                }
                (false, true) => {
                    self.visit_expression(expr.right.as_mut().unwrap())?;
                    if self.is_int {
                        let ty = expr.left.get_type();
                        let mut setter = ExpressionTypeSetter::new(ty);
                        setter.set_type_recusively(expr.right.as_mut().unwrap());
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), TypeCheckerError> {
        let variable_ty = match expr.left.as_ref() {
            Expression::Literal(l) => l.get_type(),
            _ => unreachable!("Left hand side is not a lvalue"),
        };

        self.visit_expression(&mut expr.right)?;

        if self.is_int {
            let mut setter = ExpressionTypeSetter::new(variable_ty);
            setter.set_type_recusively(&mut expr.right);
        }

        Ok(())
    }

    fn visit_array_initializer(
        &mut self,
        expr: &'ast mut ArrayInitializer,
    ) -> Result<(), TypeCheckerError> {
        self.is_int = expr.values.iter().any(|e| e.get_type().is_integer());
        Ok(())
    }
}
