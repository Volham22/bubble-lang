use libbubble::{
    ast,
    parser::{grammar::GlobalStatementsParser, lexer::Lexer},
};

fn main() {
    let lexer = Lexer::new(
        r#"
    function f() { 42 }
    function g(a: u32, b: bool) {
        let n = 32;

        while n > 21 {
            32
        }

        return n;
    }
    function h(b: bool) { 42 }
"#,
    );
    let parser = GlobalStatementsParser::new();

    match parser.parse(lexer) {
        Ok(stmts) => {
            let mut printer = ast::Printer::default();
            printer.print(stmts).expect("write to stdio failed");
        }
        Err(err) => {
            eprintln!("{err:?}");
        }
    }
}
