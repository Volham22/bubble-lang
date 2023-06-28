use libbubble::{
    ast,
    parser::{grammar::GlobalStatementsParser, lexer::Lexer},
    type_system,
};

fn main() {
    let lexer = Lexer::new(
        r#"
    function g(a: u32, b: bool): i32 {
        let n = 32;

        while n > 21 {
            32
        }

        return n;
    }

    function f(): u64 {
        let n = 42;
        return n;
    }
"#,
    );
    let parser = GlobalStatementsParser::new();

    match parser.parse(lexer) {
        Ok(mut stmts) => {
            let mut printer = ast::Printer::default();
            let mut binder = type_system::binder::Binder::default();
            let mut type_checker = type_system::type_checker::TypeChecker::default();
            let mut renamer = type_system::Renamer::default();
            binder.bind_statements(&mut stmts).expect("binder failed");
            type_checker
                .check_statements(&mut stmts)
                .expect("type checker failed");
            renamer
                .rename_statements(&mut stmts)
                .expect("renamer failed");
            printer.print(stmts).expect("write to stdio failed");
        }
        Err(err) => {
            eprintln!("{err:?}");
        }
    }
}
