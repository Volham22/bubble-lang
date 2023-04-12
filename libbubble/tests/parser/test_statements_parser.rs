use crate::parser::assets::parse_input;

use rstest::rstest;

#[rstest]
#[case::if_statement_no_else("if true { 42 }")]
#[case::if_statement_with_else("if true { 42 } else { 42 }")]
#[case::while_statement("while true { 42; }")]
#[case::for_statement("for id = 32; id != 32; id + 2 { 42 }")]
#[case::for_statement_with_typename("for id: u32 = 32; id != 32; id + 2 { 42 }")]
fn test_valid_statements(#[case] code: &str) {
    let parser_result = parse_input(code);
    assert!(
        parser_result.is_ok(),
        "Parser failed! got: {:?}",
        parser_result.unwrap_err()
    );
}
