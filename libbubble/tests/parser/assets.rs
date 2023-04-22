use libbubble::{
    ast::{GlobalStatement, Statements},
    parser::{
        grammar::{GlobalStatementsParser, StatementsParser},
        lexer::{Lexer, LexicalError, Token},
    },
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
