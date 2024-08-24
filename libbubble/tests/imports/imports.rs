use std::fs;

use libbubble::{
    ast::{GlobalStatement, ImportStatement},
    desugar::{run_imports, ImportError},
};

use crate::assets::{parse_global_statements_input, run_type_checker_with_imports};

#[test]
fn test_regular_import() {
    const MODULE_SRC: &str = "
        export function f(): void { 42; }
    ";
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let temp_file_path = temp_dir.path().join("module.blb");
    fs::write(&temp_file_path, MODULE_SRC).expect("failed to write file");

    let main_stmts = vec![GlobalStatement::Import(ImportStatement::new(
        1,
        1,
        temp_file_path.to_string_lossy().to_string(),
        Vec::new(),
    ))];

    let new_stmts = run_imports(main_stmts).expect("should not fail");
    let GlobalStatement::Function(ref f) = new_stmts[0] else {
        panic!("desugared failed");
    };
    assert_eq!(f.name, "module__f");
    assert!(f.body.is_none());
    assert!(f.is_extern);
    assert!(!f.is_exported);
}

#[test]
fn test_import_not_exists() {
    let main_stmts = vec![GlobalStatement::Import(ImportStatement::new(
        1,
        1,
        "foo.blb".to_string(),
        Vec::new(),
    ))];

    let error = run_imports(main_stmts);
    assert!(matches!(error, Err(ImportError::IOError { .. })));
}

#[test]
fn test_module_qualified_access() {
    const MAIN_SRC: &str = r#"
        import "module";

        function main(): i32 {
            module::f();
        }
    "#;

    const MODULE_SRC: &str = "
        export function f() {
            42;
        }
    ";
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let module_path = temp_dir.path().join("module.blb");
    fs::write(&module_path, MODULE_SRC).expect("failed to write module file");

    let mut stmts = parse_global_statements_input(MAIN_SRC).expect("failed to parse main file");
    // mock module path
    if let GlobalStatement::Import(i) = stmts.get_mut(0).expect("must be present") {
        i.module_path = module_path.to_string_lossy().to_string();
    } else {
        panic!("first statement is not an import statement");
    }

    run_type_checker_with_imports(stmts);
}

#[test]
fn test_qualified_import() {
    const MODULE: &str = r#"
        export function f(): i32 {
            return 0;
        }

        function g(): i32 {
            return 0;
        }
    "#;
    const MAIN: &str = r#"
        import { f } from "module";
        function main(): i32 {
            f();
            return 0;
        }
    "#;

    // Write module file
    let temp_dir = tempfile::tempdir().expect("failed to temp dir");
    let module_file = temp_dir.path().join("module.blb");
    fs::write(&module_file, MODULE).expect("failed to write");

    // Mock module file
    let mut stmts = parse_global_statements_input(MAIN).expect("parser failed");
    let GlobalStatement::Import(i) = stmts.first_mut().expect("failed to get first") else {
        panic!("first is not import");
    };
    i.module_path = module_file.to_string_lossy().to_string();
    run_type_checker_with_imports(stmts);
}

#[test]
fn test_unexistant_qualified_import() {
    const MODULE: &str = r#"
        export function f(): i32 {
            return 0;
        }

        function g(): i32 {
            return 0;
        }
    "#;
    const MAIN: &str = r#"
        import { foo } from "module";
        function main(): i32 {
            f();
            return 0;
        }
    "#;

    // Write module file
    let temp_dir = tempfile::tempdir().expect("failed to temp dir");
    let module_file = temp_dir.path().join("module.blb");
    fs::write(&module_file, MODULE).expect("failed to write");

    // Mock module file
    let mut stmts = parse_global_statements_input(MAIN).expect("parser failed");
    let GlobalStatement::Import(i) = stmts.first_mut().expect("failed to get first") else {
        panic!("first is not import");
    };
    i.module_path = module_file.to_string_lossy().to_string();
    assert!(matches!(
        run_imports(stmts),
        Err(ImportError::NotFound { .. })
    ))
}

#[test]
fn test_private_qualified_import() {
    const MODULE: &str = r#"
        export function f(): i32 {
            return 0;
        }

        function g(): i32 {
            return 0;
        }
    "#;
    const MAIN: &str = r#"
        import { g } from "module";
        function main(): i32 {
            f();
            return 0;
        }
    "#;

    // Write module file
    let temp_dir = tempfile::tempdir().expect("failed to temp dir");
    let module_file = temp_dir.path().join("module.blb");
    fs::write(&module_file, MODULE).expect("failed to write");

    // Mock module file
    let mut stmts = parse_global_statements_input(MAIN).expect("parser failed");
    let GlobalStatement::Import(i) = stmts.first_mut().expect("failed to get first") else {
        panic!("first is not import");
    };
    i.module_path = module_file.to_string_lossy().to_string();
    assert!(matches!(
        run_imports(stmts),
        Err(ImportError::NotFound { .. })
    ))
}
