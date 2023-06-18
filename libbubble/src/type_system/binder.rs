use std::collections::HashMap;

use crate::ast::{FunctionStatement, LetStatement, StructStatement, TokenLocation, Visitor};
use thiserror::Error;

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
}

pub struct Binder {
    functions_statements: HashMap<String, FunctionStatement>,
    struct_statement: HashMap<String, StructStatement>,
    local_variables: HashMap<String, LetStatement>,
}

impl Visitor<BinderError> for Binder {
}
