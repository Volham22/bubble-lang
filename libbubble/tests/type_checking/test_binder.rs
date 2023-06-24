use rstest::rstest;

use crate::assets::run_bindings;

#[rstest]
#[case::variable_binding(
    r#"
    function f() {
        let a = 2;
        return a;
    }
 "#
)]
#[case::break_in_for(
    r#"
    function f() {
        for i = 0; i < 10; 42 {
            break;
        }
    }
 "#
)]
#[case::break_in_while(
    r#"
    function f() {
        while true {
            break;
        }
    }
 "#
)]
#[case::continue_in_while(
    r#"
    function f() {
        while true {
            continue;
        }
    }
 "#
)]
#[case::continue_in_for(
    r#"
    function f() {
        for i = 0; i < 10; 42 {
            continue;
        }
    }
 "#
)]
#[case::return_in_function(
    r#"
    function f() {
        return 2;
    }
 "#
)]
#[case::struct_binding(
    r#"
    struct X {
        x: i32,
        y: bool,
    }

    function f() {
        let s: X = 32;
    }
"#
)]
#[case::function_binding(
    r#"
    function f(): u32 {
        return 42;
    }
"#
)]
#[case::recursive_function_binding(
    r#"
    function recurse() {
        recurse();
    }
"#
)]
fn test_binding_good(#[case] code: &str) {
    assert!(run_bindings(code).is_ok());
}

#[rstest]
#[case::undeclared_variable(
    r#"
    function f() {
        return var;
    }
"#
)]
#[case::out_of_scope(
    r#"
    function f(): i32 {
        while false {
            let x = 2;
        }

        return x;
    }
"#
)]
#[case::continue_not_in_loop(
    r#"
    function f() {
        continue;
    }
"#
)]
#[case::break_after_loop(
    r#"
    function f() {
        while false {
            42
        }
        break;
    }
"#
)]
#[case::continue_after_loop(
    r#"
    function f() {
        while false { 42 }
        continue;
    }
"#
)]
#[case::break_not_in_loop(
    r#"
    function f() {
        break;
    }
"#
)]
#[case::local_variable_not_callable(
    r#"
       function f() {
           let a = 2;
           a();
       }
   "#
)]
fn test_binding_bad(#[case] code: &str) {
    assert!(run_bindings(code).is_err());
}
