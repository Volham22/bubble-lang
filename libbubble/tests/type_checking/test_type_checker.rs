use libbubble::{
    ast,
    type_system::{self, TypeCheckerError},
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
        let a: i64 = 32;
        return a;
    }
"#
)]
#[case::valid_variable_init_with_type_inference(
    r#"
    function f() {
        let a: i32 = 2;
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
        for i: i32 = 2; i != 5; i = i + 1 {
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
#[case::extern_function_declaration_call(
    r#"
    extern function b(): i32;

    function f(): i64 {
        b();
        return 0;
    }
"#
)]
#[case::extern_function_declaration_call_with_args(
    r#"
    extern function b(n: i32): i32;

    function f(): i64 {
        b(32);
        return 0;
    }
"#
)]
#[case::return_int_expression_inference(
    r#"
    function main(): i64 {
        return 1 + 1;
    }
"#
)]
#[case::parameter_int_type_inference(
    r#"
    function f(x: i32): i32 {
        return x;
    }

    function main(): i32 {
        return f(0);
    }
"#
)]
#[case::identity_i32_function(
    r#"
    function f(x: i64): i64 {
        return x;
    }
"#
)]
#[case::for_int_inference_with_hint(
    r#"
    function main(): i32 {
        for i: i32 = 0; i < 5; i = i + 1 {
            42;
        }

        return 0;
    }"#
)]
#[case::for_index_iterate_array(
    r#"
    function main(): i32 {
        let arr: [3; i32] = [1, 2, 3];
        for i: i32 = 0; i < 3; i = i + 1 {
            arr[i];
        }

        return 0;
    }"#
)]
#[case::array_init_bool_inference(
    r#"
    function main(): i32 {
        let arr = [true, true, true];
        return 0;
    }"#
)]
#[case::array_init_bool_with_type(
    r#"
    function main(): i32 {
        let arr: [3; bool] = [true, true, true];
        return 0;
    }"#
)]
#[case::array_init_int_with_type(
    r#"
    function main(): i32 {
        let arr: [3; u32] = [1, 2, 3];
        return 0;
    }"#
)]
#[case::array_access(
    r#"
    function main(): i32 {
        let arr: [3; u32] = [1, 2, 3];
        arr[0];
        return 0;
    }"#
)]
#[case::array_access_function_return(
    r#"
    function main(): i32 {
        let arr: [3; i32] = [0, 2, 3];
        return arr[0];
    }"#
)]
#[case::array_as_function_parameter(
    r#"
    function f(arr: [4; u32]) {
        arr[0];
    }
    function main(): i32 {
        let arr: [3; i32] = [0, 2, 3];
        return arr[0];
    }"#
)]
#[case::array_assign(
    r#"
    function main(): i32 {
        let arr: [3; i32] = [0, 2, 3];
        arr[0] = 42;
        return 0;
    }"#
)]
#[case::pointer_init(
    r#"
    function main(): i32 {
        let arr: ptr i32 = null;
        return 0;
    }"#
)]
#[case::pass_pointer_to_function(
    r#"
    function f(x: ptr i32): i32 {
        return 42;
    }

    function main(): i32 {
        let arr: ptr i32 = null;
        f(arr);
        return 0;
    }"#
)]
#[case::function_return_pointer(
    r#"
    function f(): ptr i32 {
        return null;
    }

    function main(): i32 {
        f();
        return 0;
    }"#
)]
#[case::pointer_init_with_addrof(
    r#"
    function main(): i32 {
        let x: i32 = 42;
        let ptr_x: ptr i32 = addrof x;
        return 0;
    }"#
)]
#[case::addrof_as_function_return( // unsafe sample. For type checker purpose only
    r#"
    function f(): ptr i32 {
        let x: i32 = 42;
        return addrof x;
    }"#
)]
#[case::addrof_as_function_parameter( // unsafe sample. For type checker purpose only
    r#"
    function g(x: ptr i32): i32 {
        return 42;
    }
    function f(): ptr i32 {
        let x: i32 = 42;
        g(addrof x);
        return addrof x;
    }"#
)]
#[case::deref_ptr(
    r#"
    function f(): i32 {
        let x: ptr i32 = null;
        deref x;
        return 0;
    }"#
)]
#[case::deref_ptr_return_value(
    r#"
    function f(): i32 {
        let x: ptr i32 = null;
        return deref x;
    }"#
)]
#[case::assign_void_ptr_to_null(
    r#"
    function f(): i32 {
        let x: ptr void = null;
        return 0;
    }"#
)]
#[case::implicit_cast_to_void_ptr(
    r#"
    function f(): i32 {
        let x: i32 = 42;
        let ptr_var: ptr void = addrof x;
        return 0;
    }"#
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
#[case::extern_function_declaration_call_bad_args(
    r#"
    extern function b(n: i32): i32;

    function f(): i64 {
        b();
        return 0;
    }
    "#,
    TypeCheckerError::BadParameterCount { expected: 1, got: 0 }
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
#[case::for_int_inference_without_hint(
    r#"
    function main(): i32 {
        for i = 0; i < 5; i = i + 1 {
            42;
        }

        return 0;
    }"#,
    TypeCheckerError::InferenceError(ast::TokenLocation { line: 0, column: 0, begin: 36, end: 91 })
)]
#[case::array_init_missing_values(
    r#"
    function main(): i32 {
        let arr: [4; u32] = [1, 2, 3];
        return 0;
    }"#,
    TypeCheckerError::BadInit {
        left: type_system::Type::Array { size: 4, array_type: Box::new(type_system::Type::U32) },
        right: type_system::Type::Array { size: 3, array_type: Box::new(type_system::Type::U32) }
    },
)]
#[case::mix_type_array_init(
    r#"
    function main(): i32 {
        let arr: [3; u32] = [1, false, 3];
        return 0;
    }"#,
    TypeCheckerError::DifferentTypeInArrayInitializer { first: type_system::Type::U32, found: type_system::Type::Bool, position: 1 },
)]
#[case::wrong_type_array_init(
    r#"
    function main(): i32 {
        let arr: [3; u32] = [true, true, true];
        return 0;
    }"#,
    TypeCheckerError::BadInit { left: type_system::Type::Array {
        size: 3,
        array_type: Box::new(type_system::Type::U32),
    }, right: type_system::Type::Array {
        size: 3,
        array_type: Box::new(type_system::Type::Bool),
    } },
)]
#[case::inference_error_int_type_array_init(
    r#"
    function main(): i32 {
        let arr = [1, 2, 3];
        return 0;
    }"#,
    TypeCheckerError::InferenceError(ast::TokenLocation { line: 0, column: 0, begin: 36, end: 56 }),
)]
#[case::array_access_non_subscriptable_type(
    r#"
    function main(): i32 {
        let a: i32 = 42;
        a[0];
        return 0;
    }"#,
    TypeCheckerError::NonSubscriptable{ ty: type_system::Type::I32 },
)]
#[case::array_access_non_subscriptable_type_function_return(
    r#"
    extern function f(): i32;
    function main(): i32 {
        f()[0];
        return 0;
    }"#,
    TypeCheckerError::NonSubscriptable{ ty: type_system::Type::I32 },
)]
#[case::array_as_function_parameter_wrong_type(
    r#"
    function f(arr: [4; u32]) {
        arr[0];
    }
    function main(): i32 {
        let arr: [4; bool] = [false, false, false, false];
        f(arr);
        return 0;
    }"#,
    TypeCheckerError::BadParameter {
        name: "arr".to_string(),
        expected_type: type_system::Type::Array { size: 4, array_type: Box::new(type_system::Type::U32) },
        got: type_system::Type::Array { size: 4, array_type: Box::new(type_system::Type::Bool) } }
)]
#[case::array_as_function_parameter_good_type_wrong_size(
    r#"
    function f(arr: [4; bool]) {
        arr[0];
    }
    function main(): i32 {
        let arr: [3; bool] = [false, false, false];
        f(arr);
        return 0;
    }"#,
    TypeCheckerError::BadParameter {
        name: "arr".to_string(),
        expected_type: type_system::Type::Array { size: 4, array_type: Box::new(type_system::Type::U32) },
        got: type_system::Type::Array { size: 4, array_type: Box::new(type_system::Type::Bool) } }
)]
#[case::array_assign_bad_type(
    r#"
    function main(): i32 {
        let arr: [3; bool] = [false, false, false];
        arr[0] = 42;
        return 0;
    }"#,
    TypeCheckerError::BadAssigment {
        left: type_system::Type::Array { size: 3, array_type: Box::new(type_system::Type::Bool)},
        right: type_system::Type::Array { size: 3, array_type: Box::new(type_system::Type::U32)},
    }
)]
#[case::init_non_pointer_with_null(
    r#"
    function main(): i32 {
        let arr: i32 = null;
        return 0;
    }"#,
    TypeCheckerError::BadInit { left: type_system::Type::I32, right: type_system::Type::Null { concrete_type: None } },
)]
#[case::function_return_pointer_wrong_type(
    r#"
    function f(): ptr i32 {
        return false;
    }

    function main(): i32 {
        f();
        return 0;
    }"#,
    TypeCheckerError::ReturnTypeMismatch {
        got: type_system::Type::I32,
        expected: type_system::Type::Ptr(Box::new(type_system::Type::I32))
    } ,
)]
#[case::bad_addrof_var_init(
    r#"
    function main(): i32 {
        let x: i32 = 42;
        let ptr_x: i32 = addrof x;
        return 0;
    }"#,
    TypeCheckerError::BadInit {
        left: type_system::Type::I32,
        right: type_system::Type::Ptr(Box::new(type_system::Type::I32))
    } ,
)]
#[case::bad_addrof_parameter(
    r#"
    function g(x: ptr bool): i32 {
        return 42;
    }
    function main(): i32 {
        let x: i32 = 42;
        g(addrof x);
        return 0;
    }"#,
    TypeCheckerError::BadParameter {
        name: "x".to_string(),
        expected_type: type_system::Type::Ptr(Box::new(type_system::Type::Bool)),
        got: type_system::Type::Ptr(Box::new(type_system::Type::I32)),
    },
)]
#[case::bad_addrof_return_type(
    r#"
    function main(): i32 {
        let x: i32 = 43;
        return addrof x;
    }"#,
    TypeCheckerError::ReturnTypeMismatch {
        got: type_system::Type::Ptr(Box::new(type_system::Type::I32)),
        expected: type_system::Type::I32,
    },
)]
#[case::deref_non_ptr(
    r#"
    function main(): i32 {
        let x: i32 = 43;
        deref x;
        return 0;
    }"#,
    TypeCheckerError::DerefNonPointer(type_system::Type::I32)
)]
fn type_checker_invalid(#[case] code: &str, #[case] expected_error: TypeCheckerError) {
    let result = run_type_checker(code);

    assert!(result.is_err(), "Result should be an error");
    let err = result.unwrap_err();
    assert_eq!(
        err, expected_error,
        "got: {err:?} expected: {expected_error:?}"
    );
}
