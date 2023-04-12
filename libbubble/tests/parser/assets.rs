use libbubble::{
    ast::Statements,
    parser::{
        grammar::StatementsParser,
        lexer::{Lexer, LexicalError, Token},
    },
};

pub type StatementsParserResult =
    Result<Statements, lalrpop_util::ParseError<usize, Token, LexicalError>>;

pub fn parse_input(code: &str) -> StatementsParserResult {
    let lexer = Lexer::new(code);
    let parser = StatementsParser::new();
    parser.parse(lexer)
}
