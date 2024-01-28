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

/// Walks the source code until an other " is reached.
/// Then bump the lexer to second " location to resume lexing
/// it acts likes Flex sublexer
fn handle_quote(lex: &mut logos::Lexer<Token>) -> Result<String, ()> {
    let mut inner_content: Vec<char> = Vec::new();
    let remainder_string = lex.remainder();

    for chr in remainder_string.chars() {
        if chr == '"' {
            // Bump to the literal's size + 1 to skip the closing quote
            lex.bump(inner_content.len() + 1);
            return Ok(inner_content.iter().collect());
        }

        inner_content.push(chr);
    }

    eprint!("Error: Unclosed string literal.");
    Err(())
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
    #[token("extern")]
    Extern,

    // =================
    //       Types
    // =================

    // unsigned integers type
    #[token("u8")]
    U8Ty,
    #[token("u16")]
    U16Ty,
    #[token("u32")]
    U32Ty,
    #[token("u64")]
    U64Ty,

    // signed integer type
    #[token("i8")]
    I8Ty,
    #[token("i16")]
    I16Ty,
    #[token("i32")]
    I32Ty,
    #[token("i64")]
    I64Ty,

    // Bool, string and void
    #[token("bool")]
    BoolTy,
    #[token("string")]
    StringTy,
    #[token("void")]
    VoidTy,

    // Pointer stuff
    #[token("ptr")]
    Ptr,
    #[token("addrof")]
    Addrof,
    #[token("deref")]
    Deref,
    #[token("null")]
    Null,

    // Literals
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().parse())]
    Identifier(String),
    #[regex(r"([0-9]+)?\.[0-9]+", |lex| lex.slice().parse())]
    Real(f64),
    #[regex(r"[1-9]+[0-9]*|0", |lex| lex.slice().parse())]
    Integer(i64),
    #[token("\"", handle_quote)]
    String(String),

    #[error]
    #[regex(r"[ \r\t\v\r\n]", logos::skip)]
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
            Token::Error => {
                println!("{} {:?} {}", span.start, token, span.end);
                Err(LexicalError::InvalidToken)
            }
            _ => Ok((span.start, token, span.end)),
        })
    }
}
