use libbubble::parser::grammar::StatementsParser;
use libbubble::parser::lexer::Lexer;

fn main() {
    let lexer = Lexer::new("if true { true } else { false }");
    let parser = StatementsParser::new();

    if let Err(err) = parser.parse(lexer) {
        eprintln!("Parsing failed {:?}", err);
    } else {
        println!("Parsing OK!");
    }
}
