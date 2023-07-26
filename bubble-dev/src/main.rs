use std::{fs, path::Path, process::Command};

use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use libbubble::{
    ast::GlobalStatement,
    codegen::build_module,
    parser::{
        grammar::GlobalStatementsParser,
        lexer::{Lexer, LexicalError, Token},
    },
    type_system::{binder::Binder, type_checker::TypeChecker},
};

pub type StatementsParserResult<T> =
    Result<T, lalrpop_util::ParseError<usize, Token, LexicalError>>;

pub fn parse_global_statements_input(code: &str) -> StatementsParserResult<Vec<GlobalStatement>> {
    let lexer = Lexer::new(code);
    let parser = GlobalStatementsParser::new();
    parser.parse(lexer)
}

pub fn read_file_to_string(file: &Path) -> String {
    fs::read_to_string(file).expect("failed to read file")
}

fn main() {
    let source_code = read_file_to_string(Path::new("test.blb"));
    let mut stmts = parse_global_statements_input(&source_code).expect("Failed to parse code");
    let mut binder = Binder::default();
    let mut type_checker = TypeChecker::default();
    binder.bind_statements(&mut stmts).expect("Binder failed");
    type_checker
        .check_statements(&mut stmts)
        .expect("Type checker failed");

    let context = Context::create();
    let module = context.create_module("module");

    build_module(&context, &module, &stmts);
    Target::initialize_x86(&InitializationConfig::default());
    let target = Target::from_name("x86-64").unwrap();
    let target_machine = target
        .create_target_machine(
            &TargetMachine::get_default_triple(),
            "x86-64",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    target_machine
        .write_to_file(&module, FileType::Object, Path::new("/tmp/test.o"))
        .expect("Failed to build object file");

    let status_code = Command::new("clang")
        .arg("/tmp/test.o")
        .arg("-fPIE")
        .arg("-o")
        .arg("test")
        .status()
        .expect("Failed to invoke clang");

    assert!(
        status_code.success(),
        "ld failed with the following code: {:?}",
        status_code
    );
}
