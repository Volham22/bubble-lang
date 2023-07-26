use std::{path::Path, process::Command};

use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use libbubble::{
    ast::{GlobalStatement, Statements},
    codegen::build_module,
    parser::{
        grammar::{GlobalStatementsParser, StatementsParser},
        lexer::{Lexer, LexicalError, Token},
    },
    type_system::{binder::*, run_type_checker as type_check, TypeCheckerError},
};

const LD_LOADER_PATH: &str = "/lib64/ld-linux-x86-64.so.2";

pub type StatementsParserResult<T> =
    Result<T, lalrpop_util::ParseError<usize, Token, LexicalError>>;

pub fn parse_statements_input(code: &str) -> StatementsParserResult<Statements> {
    let lexer = Lexer::new(code);
    let parser = StatementsParser::new();
    parser.parse(lexer)
}

pub fn parse_global_statements_input(code: &str) -> StatementsParserResult<Vec<GlobalStatement>> {
    let lexer = Lexer::new(code);
    let parser = GlobalStatementsParser::new();
    parser.parse(lexer)
}

pub fn run_type_checker(code: &str) -> Result<(), TypeCheckerError> {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = Binder::default();
    binder.bind_statements(&mut stmts).expect("Binder failed");
    type_check(&mut stmts)
}

pub fn build_and_link(code: &str, outname: &str, executable_name: &str) {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = Binder::default();
    binder.bind_statements(&mut stmts).expect("Binder failed");
    type_check(&mut stmts).expect("Type checker failed");

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
        .write_to_file(&module, FileType::Object, Path::new(outname))
        .expect("Failed to build object file");

    let status_code = Command::new("ld")
        .arg("-m")
        .arg("elf_x86_64")
        .arg(outname)
        .arg("/usr/lib64/crt1.o") // C runtime
        .arg("/usr/lib64/crti.o") // C runtime
        .arg("/usr/lib64/crtn.o") // C runtime
        .arg("-lc") // Link Lib C
        // Use ld-linux-* this is needed because we're linking against the C library
        .arg("-dynamic-linker")
        .arg(LD_LOADER_PATH)
        .arg("-o")
        .arg(executable_name)
        .status()
        .expect("Failed to invoke ld");

    assert!(
        status_code.success(),
        "ld failed with the following code: {:?}",
        status_code
    );
}
