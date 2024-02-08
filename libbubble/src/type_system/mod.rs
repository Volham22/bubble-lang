pub mod binder;
mod errors;
mod inference;
mod rename;
mod typables;
mod type_checker;
mod type_setter;
mod utils;

pub use errors::{BinderError, TypeCheckerError};
pub use rename::Renamer;
pub use typables::*;
pub use type_checker::run_type_checker;
