use std::{
    convert::Infallible,
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    ast::{
        self, FunctionStatement, GlobalStatement, LiteralType, Locatable, MutableVisitor,
        TokenLocation,
    },
    parser::{self, grammar::GlobalStatementsParser, lexer::Lexer},
};

use super::mangle;

const BUBBLE_EXT: &str = "blb";

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("import failed: {file_name} {error}")]
    IOError {
        file_name: PathBuf,
        error: io::Error,
    },
    #[error("parsed failed while processing import")]
    ParserFailed {
        error: parser::ParserError,
        source_file: PathBuf,
        source_code: String,
    },
    #[error("{function_name} not found in module {module_name}")]
    NotFound {
        function_name: String,
        module_name: String,
        location: TokenLocation,
    },
}

pub type ImportResult<T> = Result<T, ImportError>;

struct DesugarQualifiedImport<'a> {
    identifier_name: &'a str,
    mangled_name: &'a str,
}

impl<'a> DesugarQualifiedImport<'a> {
    pub fn new(identifier_name: &'a str, mangled_name: &'a str) -> Self {
        Self {
            identifier_name,
            mangled_name,
        }
    }
}

impl<'ast> MutableVisitor<'ast, Infallible> for DesugarQualifiedImport<'_> {
    fn visit_literal(&mut self, expr: &'ast mut ast::Literal) -> Result<(), Infallible> {
        if let ast::LiteralType::Identifier(id) = &mut expr.literal_type {
            if id == self.identifier_name {
                *id = self.mangled_name.to_owned();
            }
        }

        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut ast::Call) -> Result<(), Infallible> {
        match &mut expr.callee.literal_type {
            LiteralType::Identifier(id) if id == self.identifier_name => {
                *id = self.mangled_name.to_string();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn process_import<T: AsRef<str>>(
    file: &Path,
    qualified: Option<&[T]>,
    location: TokenLocation,
) -> ImportResult<Vec<FunctionStatement>> {
    let file_content =
        fs::read_to_string(file.with_extension(BUBBLE_EXT)).map_err(|e| ImportError::IOError {
            file_name: file.to_path_buf(),
            error: e,
        })?;
    let lexer = Lexer::new(&file_content);
    let parser = GlobalStatementsParser::new();
    let stmts = parser
        .parse(lexer)
        .map_err(|error| ImportError::ParserFailed {
            error,
            source_file: file.with_extension(BUBBLE_EXT),
            source_code: file_content,
        })?;
    let module_name = file
        .file_stem()
        .expect("failed to extract module name")
        .to_str()
        .expect("path is not valid utf-8");

    let mut function_stmts: Vec<FunctionStatement> = stmts
        .into_iter()
        .filter_map(|stmt| match stmt {
            GlobalStatement::Function(f) if f.is_exported => Some(f),
            _ => None,
        })
        .collect();

    if let Some(qualified_list) = qualified {
        for stmt in qualified_list {
            if function_stmts.iter().all(|n| n.name != stmt.as_ref()) {
                return Err(ImportError::NotFound {
                    function_name: stmt.as_ref().to_owned(),
                    module_name: file.to_string_lossy().to_string(),
                    location,
                });
            }
        }
    }

    for stmt in function_stmts.iter_mut() {
        stmt.name = mangle::SymbolMangler::mangle_qualified(module_name, &stmt.name);

        stmt.body = None;
        stmt.is_exported = false;
        stmt.is_extern = true;
    }

    Ok(function_stmts)
}

pub fn run_imports(statements: Vec<GlobalStatement>) -> ImportResult<Vec<GlobalStatement>> {
    let mut desugared_statements = Vec::new();
    let mut desugared_symbols = Vec::new();

    for stmt in statements.into_iter() {
        if let GlobalStatement::Import(ref i) = stmt {
            let mut import_declarations = process_import(
                Path::new(&i.module_path),
                if i.elements.is_empty() {
                    None
                } else {
                    Some(&i.elements)
                },
                i.get_location().clone(),
            )?
            .into_iter()
            .map(GlobalStatement::Function)
            .collect();

            // Desugar qualified import. When using qualified import the user don't have to
            // specify explict namespace (e.g. `foo::bar()` become `bar()`). Hovewer, when building
            // the other module, the function name will be mangled. If we don't desguar these
            // identifier we will likely get a linker error with undefined symbols.
            for element_name in &i.elements {
                desugared_symbols.push((
                    PathBuf::from(i.module_path.clone()),
                    element_name.to_owned(),
                ));
            }

            desugared_statements.append(&mut import_declarations);
        } else {
            desugared_statements.push(stmt);
        }
    }

    // Apply desugar for qualified import
    for (module_path, symbol_name) in desugared_symbols {
        for stmt in desugared_statements.iter_mut() {
            if let GlobalStatement::Function(f) = stmt {
                let mangled_name = mangle::SymbolMangler::mangle_qualified(
                    &module_path
                        .file_stem()
                        .expect("failed to extract file name")
                        .to_string_lossy(),
                    &symbol_name,
                );

                let mut desugarer = DesugarQualifiedImport::new(&symbol_name, &mangled_name);
                desugarer.visit_function(f).expect("unreachable");
            }
        }
    }

    Ok(desugared_statements)
}
