use crate::ast::GlobalStatement;

use self::for_statement::desugar_for;

mod for_statement;

pub fn desugar_ast(global_statements: Vec<GlobalStatement>) -> Vec<GlobalStatement> {
    desugar_for(global_statements)
}
