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
fn test_variable_binding_good(#[case] code: &str) {
    assert!(run_bindings(code).is_ok());
}
