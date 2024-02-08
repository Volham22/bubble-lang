use std::io;

use libbubble::{
    parser::ParserError,
    type_system::{BinderError, TypeCheckerError},
};
use thiserror::Error;

pub type CompilerResult<T> = Result<T, CompilerError>;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Parser error: {0:?}")]
    Parser(ParserError),
    #[error("Binding error: {0:?}")]
    Binder(BinderError),
    #[error("Type checking error: {0:?}")]
    TypeChecker(TypeCheckerError),
    #[error("IO error: {0:?}")]
    IOError(io::Error),
    #[error("Linker error: {0}")]
    Linker(String),
}
