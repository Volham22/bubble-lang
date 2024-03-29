use crate::assets::parse_statements_input;

use rstest::rstest;

#[rstest]
#[case::bool_literal_false("false")]
#[case::bool_literal_true("true")]
#[case::float_literal("12.3")]
#[case::identifier("some_id")]
#[case::int_literal("42")]
#[case::valid_and("true and false")]
#[case::valid_and_left_assoc("true and false and true and false")]
#[case::valid_divide("1 / 1")]
#[case::valid_divide_left_assoc("1 / 1 / 1 / 2")]
#[case::valid_equal("true == true")]
#[case::valid_equal_left_assoc("true == true == true == true")]
#[case::valid_less("1 < 2")]
#[case::valid_less_identifier("a < 2")]
#[case::valid_less_equal("1 <= 2")]
#[case::valid_less_equal_left_assoc("1 <= 2 <= 3")]
#[case::valid_less_left_assoc("1 < 2 < 3")]
#[case::valid_minus("1 - 1")]
#[case::valid_minus_left_assoc("1 - 1 - 1 - 1")]
#[case::valid_modulo("1 % 1")]
#[case::valid_modulo_left_assoc("1 % 1 % 1 % 1")]
#[case::valid_more("1 > 2")]
#[case::valid_more_equal("1 >= 2")]
#[case::valid_more_equal_left_assoc("1 >= 2 >= 3")]
#[case::valid_more_left_assoc("1 > 2 > 3")]
#[case::valid_not_equal("true != false")]
#[case::valid_not_equal_left_assoc("true != false != false")]
#[case::valid_or("true or false")]
#[case::valid_or_left_assoc("true or false or true or false")]
#[case::valid_plus("1 + 1")]
#[case::valid_plus_left_assoc("1 + 1 + 1 + 1")]
#[case::valid_times("1 * 1")]
#[case::valid_times_left_assoc("1 * 1 * 1 * 1")]
#[case::simple_parenthesis("(1 + 1)")]
#[case::priority_arithmetic("(1 + 1) * 4")]
#[case::priority_logic("(true or false) and true")]
#[case::call_no_arg("f()")]
#[case::call_one_arg("f(42)")]
#[case::call_multiples_args("f(42, 3, false, true, 21 + 32)")]
#[case::call_multiples_args_trailing_comma("f(42, 3, false, true, 21 + 32,)")]
#[case::array_access("array[1]")]
#[case::array_access_with_sub_expr("array[1 + 1]")]
#[case::array_access_with_call("array[f(12)]")]
#[case::null_expr("null")]
#[case::deref("deref x")]
fn test_valid_expression(#[case] code: &str) {
    let parser_result = parse_statements_input(code);
    assert!(
        parser_result.is_ok(),
        "Parser failed! got: {:?}",
        parser_result.unwrap_err()
    );
}
