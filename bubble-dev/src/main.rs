use std::{path::Path, process::Command};

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

fn main() {
    let mut stmts = parse_global_statements_input(
        r#"    extern function puts(str: string): i32;

    function main(): i64 {
        puts("Hello, World!");
        return 0;
}"#,
    )
    .expect("Failed to parse code");
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

    let status_code = Command::new("ld")
        .arg("-m")
        .arg("elf_x86_64")
        .arg("/tmp/test.o")
        .arg("/lib64/crt1.o") // C runtime
        .arg("-lc") // Link Lib C
        // Use ld-linux-* this is needed because we're linking against the C library
        .arg("-dynamic-linker")
        .arg("/lib64/ld-linux-x86-64.so.2")
        .arg("-o")
        .arg("test")
        .status()
        .expect("Failed to invoke ld");

    assert!(
        status_code.success(),
        "ld failed with the following code: {:?}",
        status_code
    );
}
