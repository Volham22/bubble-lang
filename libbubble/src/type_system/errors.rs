use thiserror::Error;

use crate::ast::{Definition, OpType, TokenLocation};

use super::Type;

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
    #[error("Different type in array initializer. Fisrt type is: {first:?} but found {found:?} at position {position}")]
    DifferentTypeInArrayInitializer {
        first: Type,
        found: Type,
        position: u32,
    },
    #[error("Type {ty:?} is not subscriptable")]
    NonSubscriptable { ty: Type },
    #[error("Index type is not integer like. Got: {got:?}")]
    IndexNotInteger { got: Type },
    #[error("Deref a non pointer type: {0:?}.")]
    DerefNonPointer(Type),
}

impl PartialEq for TypeCheckerError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                TypeCheckerError::DerefNonPointer(_),
                TypeCheckerError::DerefNonPointer(_),
            ) | (
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
            ) | (
                TypeCheckerError::DifferentTypeInArrayInitializer { .. },
                TypeCheckerError::DifferentTypeInArrayInitializer { .. },
            ) | (
                TypeCheckerError::NonSubscriptable { .. },
                TypeCheckerError::NonSubscriptable { .. },
            )
        )
    }
}

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
    #[error("Not subscriptable expression")]
    NotSubscriptable { location: &'ast TokenLocation },
}
