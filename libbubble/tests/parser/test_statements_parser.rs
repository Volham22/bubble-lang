use rstest::rstest;

use crate::assets::{parse_global_statements_input, parse_statements_input};

#[rstest]
#[case::if_statement_no_else("if true { 42 }")]
#[case::if_statement_with_else("if true { 42 } else { 42 }")]
#[case::while_statement("while true { 42; }")]
#[case::while_statement_comparison("while a < 42 { 42; }")]
#[case::for_statement("for id = 32; id != 32; id + 2 { 42 }")]
#[case::for_statement_increment("for id = 0; id != 10; id + 1 { 32; }")]
#[case::for_statement_with_typename("for id: u32 = 32; id != 32; id + 2 { 42 }")]
#[case::valid_assignment("a = 2;")]
fn test_valid_statements(#[case] code: &str) {
    let parser_result = parse_statements_input(code);
    if parser_result.is_err() {
        match parser_result.as_ref().unwrap_err() {
            lalrpop_util::ParseError::User { error } => eprintln!("{:?}", error),
            _ => eprintln!("{:?}", parser_result.as_ref().unwrap_err()),
        }
    }

    assert!(
        parser_result.is_ok(),
        "Parser failed! got: {:?}",
        parser_result.unwrap_err()
    );
}

#[rstest]
#[case::function_no_return_type_no_parameters("function f() { 42 }")]
#[case::function_no_return_type_with_parameter("function f(a: u32) { 42 }")]
#[case::function_no_return_type_with_parameters("function f(a: u32, b: string, c: bool) { 42 }")]
#[case::function_with_return_type_with_parameter("function f(a: u32): u32 { 42 }")]
#[case::function_with_return_type_with_parameters(
    "function f(a: u32, b: string, c: bool): u32 { 42 }"
)]
#[case::function_with_assignment("function f() { x = true; }")]
#[case::function_return_type_no_parameters("function f(): u32 { 42 }")]
#[case::function_return_type_void_no_parameters("function f(): void { 42 }")]
#[case::multiple_functions(
    r#"
    function f() { 42 }
    function g(a: u32, b: bool) { 42 }
    function h(b: bool) { 42 }
"#
)]
#[case::function_with_multiple_statements(
    r#"
    function f() {
        42;
        51
    }
"#
)]
#[case::struct_one_field("struct A { a: bool }")]
#[case::struct_multiple_fields("struct A { a: bool, b: u32, c: string, }")]
#[case::struct_no_fields("struct A {}")]
#[case::let_statement_with_type("let a: u32 = 32;")]
#[case::let_statement_without_type("let a = 32;")]
#[case::return_in_function(
    r#"
    function f(): i32 {
        return 42;
    }
"#
)]
#[case::break_in_function_loop(
    r#"
    function f() {
        while true { break; }
    }
"#
)]
#[case::continue_in_function_loop(
    r#"
    function f() {
        while true { continue; }
    }
"#
)]
#[case::string_literal_variable_init(
    r#"
    function f() {
        let var = "salut";
    }
"#
)]
#[case::extern_function_declaration(
    r#"
    extern function b(): i32;

    function f(): i64 {
        return 0;
    }
"#
)]
#[case::array_declaration(
    r#"
    function f(): i64 {
        let arr = [true, false, true, true];
        return 0;
    }
"#
)]
#[case::array_declaration_with_type(
    r#"
    function f(): i64 {
        let arr: [4; u32] = [1, 2, 3, 4];
        return 0;
    }
"#
)]
fn test_valid_global_statements(#[case] code: &str) {
    let parser_result = parse_global_statements_input(code);
    assert!(
        parser_result.is_ok(),
        "Parser failed! got: {:?}",
        parser_result.unwrap_err()
    );
}
