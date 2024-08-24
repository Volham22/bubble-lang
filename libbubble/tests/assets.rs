use std::{ffi::OsStr, path::Path, process::Command};

use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use libbubble::{
    ast::{GlobalStatement, Statements},
    codegen::build_module,
    desugar::{desugar_ast, run_imports},
    parser::{
        grammar::{GlobalStatementsParser, StatementsParser},
        lexer::Lexer,
        StatementsParserResult,
    },
    type_system::{binder::*, run_type_checker as type_check, TypeCheckerError},
};

const LD_LOADER_PATH: &str = "/lib64/ld-linux-x86-64.so.2";
const MODULE_NAME: &str = "test_module";

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

pub fn run_type_checker_with_imports(mut stmts: Vec<GlobalStatement>) {
    let mut binder = Binder::default();
    stmts = run_imports(stmts).expect("import resolve failed");
    stmts = desugar_ast(stmts, "test");
    binder.bind_statements(&mut stmts).expect("Binder failed");
    type_check(&mut stmts).expect("type checker failed");
}

fn build_object(stmts: &[GlobalStatement], outname: &str) {
    let context = Context::create();
    let module = context.create_module("module");

    build_module(&context, &module, stmts, true);
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
}

fn link<T, S>(outname: T, executable_name: &str)
where
    S: AsRef<OsStr>,
    T: for<'a> Iterator<Item = S>,
{
    let status_code = Command::new("ld")
        .arg("-m")
        .arg("elf_x86_64")
        .arg("/usr/lib64/crt1.o") // C runtime
        .arg("/usr/lib64/crti.o") // C runtime
        .arg("/usr/lib64/crtn.o") // C runtime
        .args(outname)
        .arg("-lc") // Link Lib C
        // Use ld-linux-* this is needed because we're linking against the C library
        .arg("-dynamic-linker")
        .arg(LD_LOADER_PATH)
        .arg("-o")
        .arg(executable_name)
        .output()
        .expect("Failed to invoke ld");

    assert!(
        status_code.status.success(),
        "ld failed with the following code: {:?} stderr: {}",
        status_code.status,
        String::from_utf8_lossy(&status_code.stderr),
    );
}

pub fn build_objects_and_link<T: Fn(Vec<GlobalStatement>) -> Vec<GlobalStatement>>(
    objects: &[(&str, &Path)],
    executable_name: &str,
    path_ast: Option<T>,
) {
    for (code, outname) in objects {
        let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
        if let Some(f) = &path_ast {
            stmts = f(stmts);
        }
        let mut binder = Binder::new(MODULE_NAME);
        stmts = run_imports(stmts).expect("failed to process imports");
        stmts = desugar_ast(stmts, MODULE_NAME);
        binder.bind_statements(&mut stmts).expect("Binder failed");
        type_check(&mut stmts).expect("Type checker failed");

        build_object(&stmts, &outname.with_extension("o").to_string_lossy());
    }

    link(
        objects.iter().map(|(_, p)| p.with_extension("o")),
        executable_name,
    );
}

pub fn build_and_link(code: &str, outname: &str, executable_name: &str) {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = Binder::new(MODULE_NAME);
    stmts = run_imports(stmts).expect("failed to process imports");
    binder.bind_statements(&mut stmts).expect("Binder failed");
    type_check(&mut stmts).expect("Type checker failed");
    stmts = desugar_ast(stmts, MODULE_NAME);

    build_object(&stmts, outname);
    link(&mut ([outname].into_iter()), executable_name);
}
