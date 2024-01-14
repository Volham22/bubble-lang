use std::{
    io::Read,
    process::{Command, Stdio},
};

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
#[case::let_string_variable(
    r#"function main(): i32 {
        let a = "salut";
        return 0;
}"#,
    "/tmp/let_string_variable",
    0
)]
#[case::extern_function_declaration(
    r#"
    extern function b(): i32;

    function main(): i64 {
        return 0;
}"#,
    "/tmp/extern_function_declaration",
    0
)]
#[case::use_libc_puts(
    r#"
    extern function puts(str: string): i32;

    function main(): i64 {
        puts("Hello, World!");
        return 0;
}"#,
    "/tmp/use_libc_puts",
    0
)]
#[case::return_i32(
    r#"
    function main(): i32 {
        return 0;
}"#,
    "/tmp/return_i32",
    0
)]
#[case::return_i32_expression(
    r#"
    function main(): i32 {
        return 1 - 1;
}"#,
    "/tmp/return_i32_expression",
    0
)]
#[case::parameter_int_type_inference(
    r#"
    function f(x: i32): i32 {
        return x;
    }

    function main(): i32 {
        return f(0);
}"#,
    "/tmp/parameter_int_type_inference",
    0
)]
#[case::array_init(
    r#"
    function main(): i32 {
        let arr: [3; i32] = [1, 2, 3];
        return 0;
}"#,
    "/tmp/array_init",
    0
)]
fn test_translation(
    #[case] code: &str,
    #[case] executable_path: &str,
    #[case] expected_return_code: i32,
) {
    build_and_link(code, &format!("{}.o", executable_path), executable_path);

    let result = Command::new(executable_path)
        .status()
        .expect("Failed to invoke executable");
    assert_eq!(result.code().unwrap(), expected_return_code, "{:?}", result);
}

#[rstest]
#[case::while_loop_five_times(
    r#"extern function puts(msg: string): i32;
    function main(): i64 {
        let a: i64 = 0;
        while a < 5 {
            puts("hey");
            a = a + 1;
        }
        return 0;
}"#,
    "/tmp/while_loop_five_times",
    0,
    "hey\nhey\nhey\nhey\nhey\n"
)]
#[case::for_i_0_to_5(
    r#"
    extern function puts(msg: string): i32;
    function main(): i32 {
        for i: i32 = 0; i < 5; i = i + 1 {
            puts("hey");
        }

        return 0;
    }"#,
    "/tmp/for_i_0_10",
    0,
    "hey\nhey\nhey\nhey\nhey\n"
)]
#[case::for_never_loop(
    r#"
    extern function puts(msg: string): i32;
    function main(): i32 {
        for i: i32 = 0; false; i = i + 1 {
            puts("hey");
        }

        return 0;
    }"#,
    "/tmp/for_never_loop",
    0,
    ""
)]
#[case::for_condition_not_in_modify_expression(
    // i is useless here
    r#"
    extern function puts(msg: string): i32;
    function main(): i32 {
        let j: i32 = 0;
        for i: i32 = 0; j < 5; i = i + 1 {
            puts("hey");
            j = j + 1;
        }

        return 0;
    }"#,
    "/tmp/for_condition_not_in_modify_expression",
    0,
    "hey\nhey\nhey\nhey\nhey\n"
)]
#[case::for_i_is_bool(
    r#"
    extern function puts(msg: string): i32;
    function main(): i32 {
        for i: bool = true; i; i = false {
            puts("Only once");
        }

        return 0;
    }"#,
    "/tmp/for_i_is_bool",
    0,
    "Only once\n"
)]
#[case::for_variable_condition_always_false(
    r#"
    extern function puts(msg: string): i32;
    function main(): i32 {
        for i: i32 = 42; i < 10; i = i + 1 {
            puts("unreachable!");
        }

        return 0;
    }"#,
    "/tmp/for_variable_condition_always_false",
    0,
    ""
)]
fn test_translation_with_stdout(
    #[case] code: &str,
    #[case] executable_path: &str,
    #[case] expected_return_code: i32,
    #[case] expected_stdout: &str,
) {
    build_and_link(code, &format!("{}.o", executable_path), executable_path);

    let mut cmd = Command::new(executable_path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn task");

    let mut read_string = String::new();
    let mut stdout = cmd.stdout.take().expect("Failed to get executable stdout");
    let result = cmd.wait().expect("Failed to wait child process");
    stdout
        .read_to_string(&mut read_string)
        .expect("Failed to read stdout");

    assert_eq!(result.code().unwrap(), expected_return_code);
    assert_eq!(read_string, expected_stdout);
}
