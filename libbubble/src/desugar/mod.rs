use crate::ast::GlobalStatement;

use self::{for_statement::desugar_for, mangle::mangle_names};

mod for_statement;
mod mangle;

pub(crate) use mangle::SymbolMangler;

pub fn desugar_ast(
    mut global_statements: Vec<GlobalStatement>,
    module_name: &str,
) -> Vec<GlobalStatement> {
    global_statements = mangle_names(module_name, global_statements);
    desugar_for(global_statements)
}
