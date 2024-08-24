use crate::ast::GlobalStatement;

mod for_statement;
mod imports;
mod mangle;

pub use imports::{run_imports, ImportError};
pub(crate) use mangle::SymbolMangler;

pub fn desugar_ast(
    mut global_statements: Vec<GlobalStatement>,
    module_name: &str,
) -> Vec<GlobalStatement> {
    global_statements = mangle::mangle_names(module_name, global_statements);
    for_statement::desugar_for(global_statements)
}
