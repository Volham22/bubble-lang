use std::{collections::HashMap, convert::Infallible};

use crate::{
    ast::{FunctionStatement, GlobalStatement, LetStatement, Visitor},
    type_system::{Typable, Type},
};

#[derive(Debug)]
pub struct StackVariable<'a> {
    pub name: &'a str,
    pub kind: &'a Type,
}

impl<'a> StackVariable<'a> {
    pub fn new(name: &'a str, kind: &'a Type) -> Self {
        Self { name, kind }
    }
}

pub type SymbolsMap<'ast> = HashMap<&'ast str, Vec<StackVariable<'ast>>>;

#[derive(Default)]
pub struct Collector<'ast> {
    function_symbols: SymbolsMap<'ast>,
    current_function: Option<&'ast str>,
}

impl<'ast> Collector<'ast> {
    pub fn dump_global_statements(
        &mut self,
        stmts: &'ast [GlobalStatement],
    ) -> Result<&SymbolsMap<'ast>, Infallible> {
        for stmt in stmts {
            match stmt {
                GlobalStatement::Function(f) => self.visit_function(f)?,
                _ => continue,
            }
        }

        Ok(&self.function_symbols)
    }
}

impl<'ast> Visitor<'ast, Infallible> for Collector<'ast> {
    fn visit_function(&mut self, stmt: &'ast FunctionStatement) -> Result<(), Infallible> {
        if stmt.is_extern {
            return Ok(());
        }

        let collected_parameters: Vec<StackVariable<'ast>> = Vec::new();
        match stmt.get_type() {
            Type::Function { .. } => {
                self.current_function = Some(&stmt.name);
                self.function_symbols
                    .insert(&stmt.name, collected_parameters);
                self.visit_statements(stmt.body.as_ref().unwrap())?;
                self.current_function = None;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast LetStatement) -> Result<(), Infallible> {
        // Let statement can be global, leaving them into no functions. In our case we're just
        // ignoring them
        if let Some(current_function) = self.current_function {
            self.function_symbols
                .get_mut(current_function)
                .unwrap()
                .push(StackVariable::new(&stmt.name, stmt.get_type()));
        }

        Ok(())
    }
}
