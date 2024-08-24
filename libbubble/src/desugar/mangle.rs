use std::convert::Infallible;

use crate::ast::{self, MutableVisitor};

pub fn mangle_names(
    module_name: &str,
    mut global_stmts: Vec<ast::GlobalStatement>,
) -> Vec<ast::GlobalStatement> {
    let mut mangler = SymbolMangler::new(module_name);
    for stmt in global_stmts.iter_mut() {
        match mangler.visit_global_statement(stmt) {
            Ok(()) => (),
            Err(_) => unreachable!(), // name mangling is Infallible
        }
    }

    global_stmts
}

pub struct SymbolMangler<'a> {
    module_name: &'a str,
}

impl<'a> SymbolMangler<'a> {
    fn new(module_name: &'a str) -> Self {
        Self { module_name }
    }

    fn mangle_name(&self, name: &str) -> String {
        Self::mangle_qualified(self.module_name, name)
    }

    pub fn mangle_qualified(module_name: &str, identifier: &str) -> String {
        format!("{module_name}__{identifier}")
    }
}

impl<'a, 'ast> MutableVisitor<'ast, Infallible> for SymbolMangler<'a> {
    fn visit_global_statement(
        &mut self,
        stmt: &'ast mut ast::GlobalStatement,
    ) -> Result<(), Infallible> {
        match stmt {
            ast::GlobalStatement::Struct(s) => self.visit_struct(s),
            ast::GlobalStatement::Function(f) => self.visit_function(f),
            _ => Ok(()),
        }
    }

    fn visit_function(&mut self, stmt: &'ast mut ast::FunctionStatement) -> Result<(), Infallible> {
        // Do not mangle main function
        if stmt.name != "main" && !stmt.is_extern {
            stmt.name = self.mangle_name(&stmt.name);
        }

        if let Some(body) = stmt.body.as_mut() {
            self.visit_statements(body)?;
        }

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &'ast mut ast::StructStatement) -> Result<(), Infallible> {
        stmt.name = self.mangle_name(&stmt.name);
        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut ast::Literal) -> Result<(), Infallible> {
        if let ast::LiteralType::QualifiedAccess(qa) = &expr.literal_type {
            expr.literal_type = ast::LiteralType::Identifier(Self::mangle_qualified(
                &qa.module_name,
                &qa.identifier,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolMangler;

    #[test]
    fn test_name_mangle() {
        let mangler = SymbolMangler::new("foo_mod");
        assert_eq!(mangler.mangle_name("bar"), "foo_mod__bar");
    }

    #[test]
    fn test_mangle_qualified() {
        assert_eq!(SymbolMangler::mangle_qualified("foo", "bar"), "foo__bar");
    }
}
