use thiserror::Error;

use crate::ast::{Definition, OpType, TokenLocation};

use super::Type;

#[derive(Error, Debug)]
pub enum TypeCheckerError {
    #[error("{left:?} cannot be initialized with {right:?}")]
    BadInit {
        location: TokenLocation,
        left: Type,
        right: Type,
    },
    #[error("condition should be of type bool but is {1:?}")]
    NonBoolCondition(TokenLocation, Type),
    #[error("{left:?} cannot be assigned to {right:?}")]
    BadAssigment {
        location: TokenLocation,
        left: Type,
        right: Type,
    },
    #[error("{1:?} is not callable")]
    NotCallable(TokenLocation, Definition),
    #[error("Expected {expected} parameters but got {got}")]
    BadParameterCount {
        location: TokenLocation,
        expected: u32,
        got: u32,
    },
    #[error("Expected type {expected_type:?} as parameter '{name}' but got {got:?}")]
    BadParameter {
        location: TokenLocation,
        name: String,
        expected_type: Type,
        got: Type,
    },
    #[error("Cannot apply {operator:?} between {left_ty:?} and {right_ty:?}")]
    IncompatibleOperationType {
        location: TokenLocation,
        operator: OpType,
        left_ty: Type,
        right_ty: Type,
    },
    #[error("Function return type is {expected:?} but a {got:?} type is returned")]
    ReturnTypeMismatch {
        location: TokenLocation,
        got: Type,
        expected: Type,
    },
    #[error("Can't infer a proper type to the variable. Please, add a type annotation")]
    InferenceError(TokenLocation),
    #[error("Different type in array initializer. Fisrt type is: {first:?} but found {found:?} at position {position}")]
    DifferentTypeInArrayInitializer {
        location: TokenLocation,
        first: Type,
        found: Type,
        position: u32,
    },
    #[error("Type {ty:?} is not subscriptable")]
    NonSubscriptable { location: TokenLocation, ty: Type },
    #[error("Index type is not integer like. Got: {got:?}")]
    IndexNotInteger { location: TokenLocation, got: Type },
    #[error("Deref a non pointer type: {0:?}.")]
    DerefNonPointer(TokenLocation, Type),
    #[error("{1:}: Self referential struct are not allowed.")]
    SelfReferentialStruct(TokenLocation, String),
    #[error("Field '{field_name}' is no present in struct '{struct_name}'")]
    NoSuchField {
        field_name: String,
        struct_name: String,
        location: TokenLocation,
    },
    #[error("Left hand side of a field access is not a structure")]
    NonStructLhsAccess(TokenLocation),
}

impl PartialEq for TypeCheckerError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                TypeCheckerError::DerefNonPointer(..),
                TypeCheckerError::DerefNonPointer(..),
            ) | (
                TypeCheckerError::BadInit { .. },
                TypeCheckerError::BadInit { .. }
            ) | (
                TypeCheckerError::NonBoolCondition(..),
                TypeCheckerError::NonBoolCondition(..)
            ) | (
                TypeCheckerError::BadAssigment { .. },
                TypeCheckerError::BadAssigment { .. }
            ) | (
                TypeCheckerError::NotCallable(..),
                TypeCheckerError::NotCallable(..)
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
            ) | (
                TypeCheckerError::SelfReferentialStruct(..),
                TypeCheckerError::SelfReferentialStruct(..),
            )
        )
    }
}

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
    #[error("Not subscriptable expression")]
    NotSubscriptable { location: TokenLocation },
    #[error("Access expression must be a field")]
    NonIdentifierFieldAccess(TokenLocation),
}
