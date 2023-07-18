use libbubble::{
    ast,
    type_system::{self, type_checker::TypeCheckerError},
};
use rstest::rstest;

use crate::assets::run_type_checker;

#[rstest]
#[case::valid_variable_init_with_type_hint(
    r#"
    function f() {
        let a: i32 = 2;
    }
"#
)]
#[case::valid_variable_init_with_type_hint_unsigned(
    r#"
    function f() {
        let a: u32 = 0;
    }
"#
)]
#[case::void_function_return_void(
    r#"
    function f() {
        return;
    }
"#
)]
#[case::infer_type_int(
    r#"
    function f(): i64 {
        let a = 32;
        return a;
    }
"#
)]
#[case::valid_variable_init_with_type_inference(
    r#"
    function f() {
        let a = 2;
    }
"#
)]
#[case::valid_plus(
    r#"
    function f(): i32 {
        return 2 + 2;
    }
"#
)]
#[case::valid_minus(
    r#"
    function f(): i32 {
        return 2 - 2;
    }
"#
)]
#[case::valid_times(
    r#"
    function f(): i32 {
        return 2 * 2;
    }
"#
)]
#[case::valid_divide(
    r#"
    function f(): i32 {
        return 2 / 2;
    }
"#
)]
#[case::valid_modulo(
    r#"
    function f(): i32 {
        return 5 % 2;
    }
"#
)]
#[case::if_condition_is_bool(
    r#"
    function f() {
        if 1 == 2 {
            42;
        }
    }
"#
)]
#[case::while_condition_is_bool(
    r#"
    function f() {
        while 1 == 2 {
            42;
        }
    }
"#
)]
#[case::for_next_item_condition_is_bool(
    r#"
    function f() {
        for i = 2; i != 5; i = i + 1 {
            42;
        }
    }
"#
)]
#[case::let_string_type_infer(
    r#"
    function f() {
        let a = "hey";
    }
"#
)]
#[case::let_string_type_hint(
    r#"
    function f() {
        let a: string = "hey";
    }
"#
)]
#[case::function_return_string(
    r#"
    function f(): string {
        return "hey";
    }
"#
)]
#[case::function_return_variable_string(
    r#"
    function f(): string {
        let a: string = "hey";
        return a;
    }
"#
)]
fn type_checker_valid(#[case] code: &str) {
    let result = run_type_checker(code);
    assert!(
        result.is_ok(),
        "Got an unexpected type checker error {:?}",
        result.unwrap_err()
    );
}

#[rstest]
#[case::bad_binary_operation_type(r#"function f(): i32 { return 2 * false; }"#, TypeCheckerError::IncompatibleOperationType {
    operator: ast::OpType::Multiply,
    left_ty: type_system::Type::Int,
    right_ty: type_system::Type::Bool})]
#[case::bad_local_variable_init(r#"
       function f() {
           let a: bool = 32;
       }
       "#, TypeCheckerError::BadInit { left: type_system::Type::Bool, right: type_system::Type::Bool })]
#[case::bad_local_variable_assignment(r#"
       function f() {
           let a: bool = false;
           a = 42;
       }
       "#, TypeCheckerError::BadAssigment { left: type_system::Type::Bool, right: type_system::Type::Bool } )]
#[case::bad_local_variable_assignment_type_inference(r#"
       function f() {
           let a = false;
           a = 42;
       }
       "#, TypeCheckerError::BadAssigment { left: type_system::Type::Bool, right: type_system::Type::Bool } )]
#[case::condition_not_bool_while(
    r#"
       function f() {
           while 32.0 { 42 }
       }
   "#,
    TypeCheckerError::NonBoolCondition(type_system::Type::Float)
)]
#[case::condition_not_bool_if(
    r#"
       function f() {
           if 32.4 { 32 }
       }
   "#,
    TypeCheckerError::NonBoolCondition(type_system::Type::Float)
)]
#[case::condition_not_bool_if_else(
    r#"
       function f() {
           if 32.4 { 32 } else { 51 }
       }
   "#,
    TypeCheckerError::NonBoolCondition(type_system::Type::Float)
)]
#[case::condition_not_bool_for_continue_expr(
    r#"
       function f() {
           for i = 0; 32.0; i = i + 1 { 32 }
       }
   "#,
    TypeCheckerError::NonBoolCondition(type_system::Type::Float)
)]
#[case::bad_parameters_missing_args(
    r#"
       function g(a: u32, b: i32) { 32 }
       function f() {
            g();
       }
   "#,
    TypeCheckerError::BadParameterCount { expected: 0, got: 0 }
)]
#[case::bad_parameters_too_few_args(
    r#"
       function g(a: u32, b: i32) { 32 }
       function f() {
            g(2);
       }
   "#,
    TypeCheckerError::BadParameterCount { expected: 0, got: 0 }
)]
#[case::bad_parameters_arg_types_mismatch(
    r#"
       function g(a: u32, b: i32) { 32 }
       function f() {
            g(false, 32);
       }
   "#,
    TypeCheckerError::BadParameter {
        name: "".to_string(),
        expected_type: type_system::Type::U32,
        got: type_system::Type::Bool
    }
)]
#[case::incompatible_operation_type(
    r#"
       function f() {
           1 + false;
       }
   "#,
    TypeCheckerError::IncompatibleOperationType {
        operator: ast::OpType::Plus,
        left_ty: type_system::Type::I32,
        right_ty: type_system::Type::Bool
    }
)]
#[case::void_function_return_int(
    r#"
       function f() {
           return 42;
       }
   "#,
    TypeCheckerError::ReturnTypeMismatch { got: type_system::Type::Int, expected: type_system::Type::Void }
)]
#[case::wrong_int_type_return(
    r#"
       function f(): i32 {
           let a: u32 = 42;
           return a;
       }
   "#,
    TypeCheckerError::ReturnTypeMismatch { got: type_system::Type::U32, expected: type_system::Type::I32 }
)]
#[case::return_type_mismatch(
    r#"
       function f(): float {
           return 42;
       }
   "#,
    TypeCheckerError::ReturnTypeMismatch { got: type_system::Type::Int, expected: type_system::Type::Float }
)]
#[case::let_string_type_hint_bad_init(
    r#"
       function f() {
           let s: string = 32;
       }
   "#,
    TypeCheckerError::BadInit { left: type_system::Type::String, right: type_system::Type::Int }
)]
#[case::string_bad_type_assign(
    r#"
       function f() {
           let s: string = "salut";
           s = 32;
       }
   "#,
    TypeCheckerError::BadAssigment { left: type_system::Type::String, right: type_system::Type::Int }
)]
#[case::bad_return_type_string(
    r#"
       function f(): i32 {
           return "hello";
       }
   "#,
    TypeCheckerError::ReturnTypeMismatch { got: type_system::Type::String, expected: type_system::Type::I32 }
)]
fn type_checker_invalid(#[case] code: &str, #[case] expected_error: TypeCheckerError) {
    let result = run_type_checker(code);

    assert!(result.is_err(), "Result should be an error");
    assert_eq!(result.unwrap_err(), expected_error);
}
