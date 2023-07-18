use std::ops::Deref;

use thiserror::Error;

use crate::ast::{
    Assignment, BinaryOperation, Bindable, Call, Definition, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, LiteralType,
    MutableVisitor, OpType, ReturnStatement, StructStatement, WhileStatement,
};

use super::{Typable, Type};

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
        let function_type = Type::Function {
            parameters: stmt
                .parameters
                .iter()
                .map(|(kind, name)| (Type::from(kind.clone()), name.clone()))
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
        self.visit_expression(&mut stmt.init_exp)?;

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
        let fn_ty = expr.get_definition().clone();

        if let Definition::Function(ty) = fn_ty {
            if expr.arguments.len() != ty.parameters.len() {
                return Err(TypeCheckerError::BadParameterCount {
                    expected: ty.parameters.len() as u32,
                    got: expr.arguments.len() as u32,
                });
            }

            for (i, expr) in expr.arguments.iter_mut().enumerate() {
                self.visit_expression(expr)?;
                let (expected_type_kind, name) = ty.parameters.get(i).unwrap();
                let expected_type: Type = expected_type_kind.clone().into();

                if !self
                    .current_type
                    .as_ref()
                    .expect("No type in function parameter")
                    .is_compatible_with(&expected_type)
                {
                    return Err(TypeCheckerError::BadParameter {
                        name: name.clone(),
                        expected_type,
                        got: self.current_type.clone().unwrap(),
                    });
                }
            }

            Ok(())
        } else {
            Err(TypeCheckerError::NotCallable(fn_ty))
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
                println!("{left_ty:?} {:?}", expr.left);

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
                Ok(())
            }
            LiteralType::Integer(_) => {
                self.current_type = Some(Type::Int);
                Ok(())
            }
            LiteralType::Float(_) => {
                self.current_type = Some(Type::Float);
                Ok(())
            }
            LiteralType::String(_) => {
                self.current_type = Some(Type::String);
                Ok(())
            }
            LiteralType::Identifier(_) => {
                // FIXME: This is ugly and should not be written this way. We're
                // cloning here to trick the borrow checker and do mutable accept
                match literal.get_definition().clone() {
                    Definition::Struct(ref mut strct) => {
                        self.visit_struct(strct)?;
                        literal.set_type(self.current_type.as_ref().unwrap().clone());
                        Ok(())
                    }
                    Definition::LocalVariable(ref mut v) => {
                        self.visit_let(v)?;
                        literal.set_type(self.current_type.as_ref().unwrap().clone());
                        Ok(())
                    }
                    Definition::Function(ref mut f) => {
                        self.visit_function(f)?;
                        literal.set_type(self.current_type.as_ref().unwrap().clone());
                        Ok(())
                    }
                }
            }
        }
    }
}
