use std::ops::Deref;

use thiserror::Error;

use crate::ast::{
    Assignment, BinaryOperation, Bindable, Call, Definition, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, LiteralType,
    MutableVisitor, OpType, ReturnStatement, StructStatement, TokenLocation, WhileStatement,
};

use super::{inference::IntegerInference, Typable, Type};

pub fn run_type_checker(stmts: &mut [GlobalStatement]) -> Result<(), TypeCheckerError> {
    let mut type_checker = TypeChecker::default();
    let mut int_inference = IntegerInference::default();

    type_checker.check_statements(stmts)?;
    int_inference.infer_statements(stmts)?;

    // type_checker.check_statements(stmts)
    Ok(())
}

#[derive(Error, Debug)]
pub enum TypeCheckerError {
    #[error("{left:?} cannot be initialized with {right:?}")]
    BadInit { left: Type, right: Type },
    #[error("condition should be of type bool but is {0:?}")]
    NonBoolCondition(Type),
    #[error("{left:?} cannot be assigned to {right:?}")]
    BadAssigment { left: Type, right: Type },
    #[error("{0:?} is not callable")]
    NotCallable(Definition),
    #[error("Expected {expected} parameters but got {got}")]
    BadParameterCount { expected: u32, got: u32 },
    #[error("Expected type {expected_type:?} as parameter '{name}' but got {got:?}")]
    BadParameter {
        name: String,
        expected_type: Type,
        got: Type,
    },
    #[error("Cannot apply {operator:?} between {left_ty:?} and {right_ty:?}")]
    IncompatibleOperationType {
        operator: OpType,
        left_ty: Type,
        right_ty: Type,
    },
    #[error("Function return type is {expected:?} but a {got:?} type is returned")]
    ReturnTypeMismatch { got: Type, expected: Type },
    #[error("Can't infer a proper type to the variable. Please, add a type annotation")]
    InferenceError(TokenLocation),
}

impl PartialEq for TypeCheckerError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                TypeCheckerError::BadInit { .. },
                TypeCheckerError::BadInit { .. }
            ) | (
                TypeCheckerError::NonBoolCondition(_),
                TypeCheckerError::NonBoolCondition(_)
            ) | (
                TypeCheckerError::BadAssigment { .. },
                TypeCheckerError::BadAssigment { .. }
            ) | (
                TypeCheckerError::NotCallable(_),
                TypeCheckerError::NotCallable(_)
            ) | (
                TypeCheckerError::BadParameterCount { .. },
                TypeCheckerError::BadParameterCount { .. },
            ) | (
                TypeCheckerError::BadParameter { .. },
                TypeCheckerError::BadParameter { .. }
            ) | (
                TypeCheckerError::IncompatibleOperationType { .. },
                TypeCheckerError::IncompatibleOperationType { .. },
            ) | (
                TypeCheckerError::ReturnTypeMismatch { .. },
                TypeCheckerError::ReturnTypeMismatch { .. },
            ) | (
                TypeCheckerError::InferenceError(_),
                TypeCheckerError::InferenceError(_),
            )
        )
    }
}

#[derive(Default)]
pub struct TypeChecker {
    current_type: Option<Type>,
    current_function: Option<Type>, // current's function type
}

impl<'ast> TypeChecker {
    pub fn check_statements(
        &mut self,
        stmts: &'ast mut [GlobalStatement],
    ) -> Result<(), TypeCheckerError> {
        for stmt in stmts.iter_mut() {
            self.visit_global_statement(stmt)?;
            self.current_type = None;
            self.current_function = None;
        }

        Ok(())
    }

    fn check_bool_expression(
        &mut self,
        expr: &'ast mut Expression,
    ) -> Result<(), TypeCheckerError> {
        self.visit_expression(expr)?;

        match self
            .current_type
            .as_ref()
            .expect("expression should have type")
        {
            Type::Bool => Ok(()),
            _ => Err(TypeCheckerError::NonBoolCondition(
                self.current_type.clone().unwrap(),
            )),
        }
    }
}

impl<'ast> MutableVisitor<'ast, TypeCheckerError> for TypeChecker {
    fn visit_function(
        &mut self,
        stmt: &'ast mut FunctionStatement,
    ) -> Result<(), TypeCheckerError> {
        // Set parameters type
        for parameter in stmt.parameters.iter_mut() {
            parameter.set_type(Type::from(
                parameter
                    .declaration_type
                    .clone()
                    .expect("Parameter has no type hint!"),
            ))
        }

        let function_type = Type::Function {
            parameters: stmt
                .parameters
                .iter()
                .map(|let_stmt| (let_stmt.get_type().clone(), let_stmt.name.clone()))
                .collect(),
            return_type: Box::new(stmt.return_type.clone().into()),
        };

        self.current_function = Some(function_type.clone());
        stmt.set_type(function_type);

        if let Some(body) = stmt.body.as_mut() {
            self.visit_statements(body)?;
        }

        self.current_function = None;
        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), TypeCheckerError> {
        if let Some(ref mut exp) = stmt.exp {
            self.visit_expression(exp)?;
        } else {
            self.current_type = Some(Type::Void);
        }

        match self
            .current_function
            .as_ref()
            .expect("return outside a function")
        {
            Type::Function { return_type, .. } => {
                if !return_type.is_compatible_with(
                    self.current_type
                        .as_ref()
                        .expect("return expression has no type"),
                ) {
                    Err(TypeCheckerError::ReturnTypeMismatch {
                        got: self.current_type.clone().unwrap(),
                        expected: return_type.deref().clone(),
                    })
                } else {
                    Ok(())
                }
            }
            _ => unreachable!("current function type is not a function!"),
        }
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), TypeCheckerError> {
        stmt.set_type(Type::Struct {
            name: stmt.name.clone(),
            fields: stmt
                .fields
                .iter()
                .map(|(kind, name)| (Type::from(kind.clone()), name.clone()))
                .collect(),
        });

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), TypeCheckerError> {
        self.visit_expression(
            stmt.init_exp
                .as_mut()
                .expect("Let statement has no init exp!"),
        )?;

        match &stmt.declaration_type {
            Some(ty) => {
                let real_type: Type = ty.clone().into();

                if !real_type
                    .is_compatible_with(self.current_type.as_ref().expect("let init has no type"))
                {
                    return Err(TypeCheckerError::BadInit {
                        left: real_type,
                        right: self.current_type.clone().unwrap(),
                    });
                }

                self.current_type = Some(real_type.clone());
                stmt.set_type(real_type);
            }
            None => {
                stmt.set_type(self.current_type.as_ref().unwrap().clone());
            }
        }

        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast mut IfStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.then_clause)?;

        if let Some(stmts) = &mut stmt.else_clause {
            self.visit_statements(stmts)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast mut WhileStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast mut ForStatement) -> Result<(), TypeCheckerError> {
        self.visit_let(&mut stmt.init_decl)?;
        self.check_bool_expression(&mut stmt.continue_expression)?;
        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), TypeCheckerError> {
        self.visit_expression(&mut expr.left)?;

        let lhs_ty = self
            .current_type
            .clone()
            .expect("left expression should have a type");

        self.visit_expression(&mut expr.right)?;

        let rhs_ty = self
            .current_type
            .as_ref()
            .expect("left expression should have a type");

        if !lhs_ty.is_compatible_with(rhs_ty) {
            return Err(TypeCheckerError::BadAssigment {
                left: lhs_ty,
                right: rhs_ty.clone(),
            });
        }

        self.current_type = Some(lhs_ty);

        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), TypeCheckerError> {
        if expr.get_definition().is_function() {
            if expr.arguments.len() != expr.get_function_def().parameters.len() {
                return Err(TypeCheckerError::BadParameterCount {
                    expected: expr.get_function_def().parameters.len() as u32,
                    got: expr.arguments.len() as u32,
                });
            }

            // Add parameters types to a vector
            let mut parameter_types = Vec::with_capacity(expr.get_function_def().parameters.len());
            for param_expr in expr.arguments.iter_mut() {
                self.visit_expression(param_expr)?;
                parameter_types.push(
                    self.current_type
                        .clone()
                        .expect("Parameter expression should be typed"),
                );
            }

            for (expr_type, function_parameter) in parameter_types
                .iter()
                .zip(expr.get_function_def().parameters.iter())
            {
                let expected_type = function_parameter
                    .ty
                    .clone()
                    .expect("Parameter should be typed");

                if !expr_type.is_compatible_with(&expected_type) {
                    return Err(TypeCheckerError::BadParameter {
                        name: function_parameter.name.clone(),
                        expected_type,
                        got: self.current_type.clone().unwrap(),
                    });
                }
            }

            Ok(())
        } else {
            Err(TypeCheckerError::NotCallable(expr.get_definition().clone()))
        }
    }

    fn visit_type(&mut self, ty: &'ast mut crate::ast::Type) -> Result<(), TypeCheckerError> {
        self.current_type = Some(ty.kind.clone().into());
        Ok(())
    }

    fn visit_binary_operation(
        &mut self,
        expr: &'ast mut BinaryOperation,
    ) -> Result<(), TypeCheckerError> {
        match expr.right {
            // Binary operation
            Some(ref mut right_exp) => {
                self.visit_expression(&mut expr.left)?;
                let left_ty = self
                    .current_type
                    .clone()
                    .expect("No Left type in binary operation!");

                self.visit_expression(right_exp)?;

                let right_ty = self
                    .current_type
                    .as_ref()
                    .expect("No right type in binary operation!");

                if !left_ty.is_compatible_with(right_ty) {
                    return Err(TypeCheckerError::IncompatibleOperationType {
                        operator: expr.op,
                        left_ty,
                        right_ty: self.current_type.clone().unwrap(),
                    });
                }

                // Plus, Minus, Multiply, Divide and modulo expression has a result of their type
                if matches!(
                    expr.op,
                    OpType::Plus
                        | OpType::Minus
                        | OpType::Multiply
                        | OpType::Divide
                        | OpType::Modulo
                ) {
                    self.current_type = Some(right_ty.clone());
                } else {
                    self.current_type = Some(Type::Bool);
                }

                Ok(())
            }
            // Unary operation
            None => match expr.op {
                OpType::Minus => {
                    self.visit_expression(&mut expr.left)?;
                    Ok(())
                }
                OpType::Not => {
                    self.visit_expression(&mut expr.left)?;
                    self.current_type = Some(Type::Bool);
                    Ok(())
                }
                // This is a bug, and should never happen
                _ => unreachable!("Unary operation should be `not` or `-`"),
            },
        }
    }

    fn visit_literal(&mut self, literal: &'ast mut Literal) -> Result<(), TypeCheckerError> {
        match literal.literal_type {
            LiteralType::True | LiteralType::False => {
                self.current_type = Some(Type::Bool);
                literal.set_type(Type::Bool);
            }
            LiteralType::Integer(_) => {
                self.current_type = Some(Type::Int);
                literal.set_type(Type::Int);
            }
            LiteralType::Float(_) => {
                self.current_type = Some(Type::Float);
                literal.set_type(Type::Float);
            }
            LiteralType::String(_) => {
                self.current_type = Some(Type::String);
                literal.set_type(Type::String);
            }
            LiteralType::Identifier(_) => {
                // FIXME: This is ugly and should not be written this way. We're
                // cloning here to trick the borrow checker and do mutable accept
                match literal.get_definition().clone() {
                    Definition::Struct(_) => {
                        let strct = literal.get_struct_def();
                        // self.visit_struct(strct)?;
                        self.current_type = Some(strct.get_type().clone());
                        literal.set_type(strct.get_type().clone());
                    }
                    Definition::LocalVariable(_) => {
                        self.current_type =
                            Some(literal.get_local_variable_def().get_type().clone());
                        literal.set_type(literal.get_local_variable_def().get_type().clone());
                    }
                    Definition::Function(_) => {
                        self.current_type = Some(literal.get_function_def().get_type().clone());
                        literal.set_type(literal.get_function_def().get_type().clone());
                    }
                }
            }
            LiteralType::ArrayAccess(_) => todo!(),
        };

        Ok(())
    }
}
