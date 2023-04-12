use std::fmt;

use logos::{Logos, SpannedIter};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
    InvalidToken,
    InvalidIntegerLiteral { msg: String },
    InvalidFloatLiteral { msg: String },
}

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Syntax elements
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftCurlyBracket,
    #[token("}")]
    RightCurlyBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("=")]
    Equal,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    Less,
    #[token(">")]
    More,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    MoreEqual,

    // Keywords
    #[token("function")]
    Function,
    #[token("struct")]
    Struct,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("let")]
    Let,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // =================
    //       Types
    // =================

    // unsigned integers type
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,

    // signed integer type
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,

    // Bool and string
    #[token("bool")]
    Bool,
    #[token("string")]
    String,

    // Literals
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().parse())]
    Identifier(String),
    #[regex(r"([0-9]+)?\.[0-9]+", |lex| lex.slice().parse())]
    Real(f64),
    #[regex(r"[1-9]+[0-9]*", |lex| lex.slice().parse())]
    Integer(i64),

    #[error]
    #[regex(r"[ \r\t\v\r]", logos::skip)]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| match token {
            Token::Error => Err(LexicalError::InvalidToken),
            _ => Ok((span.start, token, span.end)),
        })
    }
}
