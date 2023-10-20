pub mod binder;
mod inference;
mod rename;
mod typables;
mod type_checker;
mod utils;

pub use rename::Renamer;
pub use typables::*;
pub use type_checker::{run_type_checker, TypeCheckerError};
