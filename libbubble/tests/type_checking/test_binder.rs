use rstest::rstest;

use libbubble::type_system::binder;

use crate::assets::parse_global_statements_input;

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
#[case::array_usage(
    r#"
       function f() {
           let a = [true, false, true];
           a[0];
       }
   "#
)]
#[case::array_usage_lvalue_call(
    r#"
       extern function g(): u32;

       function f() {
           g()[0];
       }
   "#
)]
#[case::nested_struct_access(
    r#"
    struct Other {
        y: i32,
        z: i32,
    }

    struct Point {
        x: i32,
        y: Other,
    }

    function main(): i32 {
        let p: Point = struct {
            x: 42,
            y: struct { y: 51, z: 42 },
        };

        p.y.z;
        return 0;
}"#
)]
#[case::access_field_of_deref(
    r#"
    struct Point {
        x: i32,
        y: i32,
    }

    function f(): i32 {
        let p: Point = struct { x: 0, y: 0 };
        let ptr_p: ptr Point = addrof p;
        return (deref ptr_p).x;
    }
    "#
)]
#[case::deref_field_of_parameter_ptr(
    r#"
    struct Point {
        x: i32,
        y: i32,
    }

    function main(): i32 {
        let p: Point = struct { x: 0, y: 42 };
        let ptr_p: ptr Point = addrof p;
        return (deref ptr_p).x;
    }
    "#
)]
fn test_binding_good(#[case] code: &str) {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = binder::Binder::default();
    let result = binder.bind_statements(&mut stmts);
    assert!(result.is_ok(), "got: {:?}", result.unwrap_err());
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
#[case::expr_subscriptable(
    r#"
       function f() {
           42[0];
       }
   "#
)]
fn test_binding_bad(#[case] code: &str) {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = binder::Binder::default();
    assert!(binder.bind_statements(&mut stmts).is_err());
}
