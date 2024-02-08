use std::path::{Path, PathBuf};

use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use libbubble::{
    ast,
    codegen::build_module,
    desugar::desugar_ast,
    parser::{grammar::GlobalStatementsParser, lexer::Lexer},
    type_system::{self, binder::Binder},
};

use crate::{
    errors::{CompilerError, CompilerResult},
    io::load_source_file,
};

fn parse_source_code(source_code: &str) -> CompilerResult<Vec<ast::GlobalStatement>> {
    let lexer = Lexer::new(source_code);
    let parser = GlobalStatementsParser::new();

    parser.parse(lexer).map_err(CompilerError::Parser)
}

fn run_type_checker(stmts: &mut [ast::GlobalStatement]) -> CompilerResult<()> {
    let mut binder = Binder::default();
    binder
        .bind_statements(stmts)
        .map_err(CompilerError::Binder)?;

    type_system::run_type_checker(stmts).map_err(CompilerError::TypeChecker)
}

fn build_object(
    source_code: &str,
    object_name: &Path,
    print_llvmir: bool,
    emit_llvmir: bool,
) -> CompilerResult<()> {
    let mut stmts = parse_source_code(source_code)?;
    run_type_checker(&mut stmts)?;
    let desugared_stmts = desugar_ast(stmts);
    let llvm_context = Context::create();
    let llvm_module = llvm_context.create_module(
        object_name
            .file_name()
            .expect("Failed to extract filename")
            .to_str()
            .expect("Failed to convert to str"),
    );

    build_module(&llvm_context, &llvm_module, &desugared_stmts, print_llvmir);

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
        .write_to_file(&llvm_module, FileType::Object, Path::new(object_name))
        .expect("Failed to build object file");

    if emit_llvmir {
        llvm_module
            .print_to_file(format!(
                "{}.ll",
                object_name
                    .file_stem()
                    .expect("Failed to extract stem")
                    .to_str()
                    .expect("Failed to convert to str")
            ))
            .expect("Failed to emit llvm ir");
    }

    Ok(())
}

pub fn build_objects_targets(
    targets: &[&Path],
    print_llvmir: bool,
    emit_llvmir: bool,
) -> CompilerResult<Vec<PathBuf>> {
    let mut built_objects = Vec::new();
    for source_code_path in targets
        .iter()
        .filter(|p| p.extension().expect("Failed to extract extension") == "blb")
    {
        let source_code = load_source_file(source_code_path).map_err(CompilerError::IOError)?;
        let object_path = PathBuf::from(format!(
            "{}.o",
            source_code_path
                .file_name()
                .expect("Failed to extract file name")
                .to_str()
                .expect("Failed to convert to str")
        ));
        build_object(&source_code, &object_path, print_llvmir, emit_llvmir)?;
        built_objects.push(object_path);
    }

    Ok(built_objects)
}
