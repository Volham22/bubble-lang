use thiserror::Error;

use crate::ast::{
    Assignment, BinaryOperation, Bindable, Call, Definition, Expression, ForStatement,
    FunctionStatement, IfStatement, LetStatement, Literal, LiteralType, MutableVisitor, OpType,
    Statements, StructStatement, WhileStatement,
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
    #[error("Typename {0} does not name a structure!")]
    TypenameNotStructure(String),
}

#[derive(Default)]
pub struct TypeChecker {
    current_type: Option<Type>,
    current_function: Option<Type>, // current's function type
}

impl TypeChecker {
    fn visit_statements(&mut self, stmts: &mut Statements) -> Result<(), TypeCheckerError> {
        for stmt in stmts.statements.iter_mut() {
            stmt.kind.accept_mut(self)?;
        }

        Ok(())
    }

    fn check_bool_expression(&mut self, expr: &mut Expression) -> Result<(), TypeCheckerError> {
        expr.accept_mut(self)?;

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

impl MutableVisitor<TypeCheckerError> for TypeChecker {
    fn visit_function(&mut self, stmt: &mut FunctionStatement) -> Result<(), TypeCheckerError> {
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

        for function_stmt in stmt.body.statements.iter_mut() {
            function_stmt.kind.accept_mut(self)?;
        }

        self.current_function = None;
        Ok(())
    }

    fn visit_struct(&mut self, stmt: &mut StructStatement) -> Result<(), TypeCheckerError> {
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

    fn visit_let(&mut self, stmt: &mut LetStatement) -> Result<(), TypeCheckerError> {
        stmt.init_exp.accept_mut(self)?;
        match &stmt.declaration_type {
            // Type is defined in code let's check if the variable can be initialized
            // with the left expression
            Some(ty) => {
                let real_type: Type = ty.clone().into();
                if !real_type
                    .is_compatible_with(self.current_type.as_ref().expect("let init has no type"))
                {
                    stmt.set_type(real_type.clone());

                    return Err(TypeCheckerError::BadInit {
                        left: real_type,
                        right: self.current_type.clone().unwrap(),
                    });
                }
            }
            // Infer the variable type with the right hand side expression type
            None => {
                stmt.set_type(self.current_type.clone().expect("let init has not type"));
            }
        }

        Ok(())
    }

    fn visit_if(&mut self, stmt: &mut IfStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.then_clause)?;

        if let Some(stmts) = &mut stmt.else_clause {
            self.visit_statements(stmts)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &mut WhileStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_for(&mut self, stmt: &mut ForStatement) -> Result<(), TypeCheckerError> {
        stmt.init_decl.accept_mut(self)?;
        self.check_bool_expression(&mut stmt.continue_expression)?;
        stmt.modify_expression.accept_mut(self)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_assignment(&mut self, expr: &mut Assignment) -> Result<(), TypeCheckerError> {
        expr.left.accept_mut(self)?;
        let lhs_ty = self
            .current_type
            .clone()
            .expect("left expression should have a type");

        expr.right.accept_mut(self)?;
        let rhs_ty = self
            .current_type
            .clone()
            .expect("left expression should have a type");

        if !lhs_ty.is_compatible_with(&rhs_ty) {
            return Err(TypeCheckerError::BadAssigment {
                left: lhs_ty,
                right: rhs_ty,
            });
        }

        self.current_type = Some(lhs_ty);

        Ok(())
    }

    fn visit_call(&mut self, expr: &mut Call) -> Result<(), TypeCheckerError> {
        let fn_ty = expr.get_definition().clone();

        if let Definition::Function(ty) = fn_ty {
            if expr.arguments.len() != ty.parameters.len() {
                return Err(TypeCheckerError::BadParameterCount {
                    expected: ty.parameters.len() as u32,
                    got: expr.arguments.len() as u32,
                });
            }

            for (i, expr) in expr.arguments.iter_mut().enumerate() {
                expr.accept_mut(self)?;
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
            Err(TypeCheckerError::NotCallable(fn_ty.clone()))
        }
    }

    fn visit_type(&mut self, ty: &mut crate::ast::Type) -> Result<(), TypeCheckerError> {
        self.current_type = Some(ty.kind.clone().into());
        Ok(())
    }

    fn visit_binary_operation(
        &mut self,
        expr: &mut BinaryOperation,
    ) -> Result<(), TypeCheckerError> {
        match expr.right {
            // Binary operation
            Some(ref mut right_exp) => {
                expr.left.accept_mut(self)?;
                let left_ty = self
                    .current_type
                    .clone()
                    .expect("No Left type in binary operation!");

                right_exp.accept_mut(self)?;

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
                    self.current_type = Some(Type::Bool);
                } else {
                    self.current_type = Some(right_ty.clone());
                }

                Ok(())
            }
            // Unary operation
            None => match expr.op {
                OpType::Minus => {
                    expr.left.accept_mut(self)?;
                    Ok(())
                }
                OpType::Not => {
                    expr.left.accept_mut(self)?;
                    self.current_type = Some(Type::Bool);
                    Ok(())
                }
                // This is a bug, and should never happen
                _ => unreachable!("Unary operation should be `not` or `-`"),
            },
        }
    }

    fn visit_literal(&mut self, literal: &mut Literal) -> Result<(), TypeCheckerError> {
        match literal.literal_type {
            LiteralType::True | LiteralType::False => {
                self.current_type = Some(Type::Bool);
                Ok(())
            }
            LiteralType::Integer(_) => {
                self.current_type = Some(Type::I32);
                Ok(())
            }
            LiteralType::Float(_) => {
                self.current_type = Some(Type::Float);
                Ok(())
            }
            LiteralType::Identifier(ref id) => {
                // FIXME: This is ugly and should not be written this way.
                if let Definition::Struct(mut strct) = literal.get_definition().clone() {
                    strct.accept_mut(self)?;
                    Ok(())
                } else {
                    Err(TypeCheckerError::TypenameNotStructure(id.to_string()))
                }
            }
        }
    }
}
