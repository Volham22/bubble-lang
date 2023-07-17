use std::process::Command;

use rstest::rstest;

use crate::assets::build_and_link;

#[rstest]
#[case::main_return_0(
    r#"function main(): i64 {
    return 0;
}"#,
    "/tmp/return_0",
    0
)]
#[case::main_return_42(
    r#"function main(): i64 {
    return 42;
}"#,
    "/tmp/return_42",
    42
)]
#[case::if_without_else(
    r#"function main(): i64 {
        if 2 > 1 {
            return 42;
        }

        return 0;
}"#,
    "/tmp/if_without_else",
    42
)]
#[case::if_else_branch_then(
    r#"function main(): i64 {
        if 2 > 1 {
            return 42;
        } else {
            return 0;
        }

        return 0;
}"#,
    "/tmp/if_else_branch_then",
    42
)]
#[case::if_else_branch_else(
    r#"function main(): i64 {
        if 1 > 2 {
            return 0;
        } else {
            return 42;
        }

        return 0;
}"#,
    "/tmp/if_else_branch_else",
    42
)]
#[case::let_statement_init(
    r#"function main(): i64 {
        let a: i64 = 42;
        return a;
}"#,
    "/tmp/let_statement_init",
    42
)]
#[case::assign_variable(
    r#"function main(): i64 {
        let a: i64 = 42;
        a = 0;
        return a;
}"#,
    "/tmp/assign_variable",
    0
)]
#[case::main_return_void(r#"function main() { return; }"#, "/tmp/return_void", 0)]
fn test_translation(
    #[case] code: &str,
    #[case] executable_path: &str,
    #[case] expected_return_code: i32,
) {
    build_and_link(code, &format!("{}.o", executable_path), executable_path);

    let result = Command::new(executable_path)
        .status()
        .expect("Failed to invoke executable");
    assert_eq!(result.code().unwrap(), expected_return_code);
}
