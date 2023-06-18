use libbubble::{
    ast::{GlobalStatement, Statements},
    parser::{
        grammar::{GlobalStatementsParser, StatementsParser},
        lexer::{Lexer, LexicalError, Token},
    },
    type_system::binder::*,
};

pub type StatementsParserResult<T> =
    Result<T, lalrpop_util::ParseError<usize, Token, LexicalError>>;

pub fn parse_statements_input(code: &str) -> StatementsParserResult<Statements> {
    let lexer = Lexer::new(code);
    let parser = StatementsParser::new();
    parser.parse(lexer)
}

pub fn parse_global_statements_input(code: &str) -> StatementsParserResult<Vec<GlobalStatement>> {
    let lexer = Lexer::new(code);
    let parser = GlobalStatementsParser::new();
    parser.parse(lexer)
}

pub fn run_bindings(code: &str) -> Result<(), BinderError> {
    let mut stmts = parse_global_statements_input(code).expect("Failed to parse code");
    let mut binder = Binder::default();
    binder.bind_statements(&mut stmts)
}
