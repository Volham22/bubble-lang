use libbubble::parser::lexer::Token;
use logos::Logos;
use rstest::rstest;

#[rstest]
#[case::parenthesis("()", &[Token::LeftParen, Token::RightParen])]
#[case::braces("[]", &[Token::LeftBracket, Token::RightBracket])]
#[case::curly_braces("{}", &[Token::LeftCurlyBracket, Token::RightCurlyBracket])]
#[case::commas(", ,", &[Token::Comma, Token::Comma])]
#[case::semicolon("; ;", &[Token::Semicolon, Token::Semicolon])]
#[case::colon(": :", &[Token::Colon, Token::Colon])]
#[case::equal("= =", &[Token::Equal, Token::Equal])]
#[case::one_plus_one("1 + 1", &[Token::Integer(1), Token::Plus, Token::Integer(1)])]
#[case::one_minus_one("1 - 1", &[Token::Integer(1), Token::Minus, Token::Integer(1)])]
#[case::one_times_one("1 * 1", &[Token::Integer(1), Token::Star, Token::Integer(1)])]
#[case::one_divide_one("1 / 1", &[Token::Integer(1), Token::Slash, Token::Integer(1)])]
#[case::and("true and false", &[Token::True, Token::And, Token::False])]
#[case::or("true or false", &[Token::True, Token::Or, Token::False])]
#[case::equality("true == false", &[Token::True, Token::EqualEqual, Token::False])]
#[case::different("true != false", &[Token::True, Token::BangEqual, Token::False])]
#[case::while_kw(
    "while false {}",
    &[
        Token::While,
        Token::False,
        Token::LeftCurlyBracket,
        Token::RightCurlyBracket
    ]
)]
#[case::break_kw("break break", &[Token::Break, Token::Break])]
#[case::continue_kw("continue continue", &[Token::Continue, Token::Continue])]
#[case::true_kw("true true", &[Token::True, Token::True])]
#[case::true_kw("false false", &[Token::False, Token::False])]
#[case::for_kw("for for", &[Token::For, Token::For])]
#[case::type_unsigned("u8 u16 u32 u64", &[Token::U8, Token::U16, Token::U32, Token::U64])]
#[case::type_unsigned("i8 i16 i32 i64", &[Token::I8, Token::I16, Token::I32, Token::I64])]
#[case::identifier("my_var", &[Token::Identifier("my_var".to_string())])]
#[case::integer("42", &[Token::Integer(42)])]
#[case::integer_too_big(
    "38574895743859734589347589347598340853495873409584389573489574389574389573",
    &[Token::Error]
)]
#[allow(clippy::approx_constant)] // it's ok for a lexer test
#[case::float("3.14", &[Token::Real(3.14)])]
#[case::nothing_before_dot_float(".032", &[Token::Real(0.032)])]
#[case::let_declaration(
    "let var = 42;",
    &[
        Token::Let,
        Token::Identifier("var".to_string()),
        Token::Equal, Token::Integer(42),
        Token::Semicolon
    ]
)]
#[case::operators(
    "< > <= >= == !=",
    &[
        Token::Less,
        Token::More,
        Token::LessEqual,
        Token::MoreEqual,
        Token::EqualEqual,
        Token::BangEqual
    ]
)]
#[case::multiline_code(
    r#"
        function f() { 42 }
        function g() { 42 }
    "#,
    &[
        Token::Function,
        Token::Identifier("f".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftCurlyBracket,
        Token::Integer(42),
        Token::RightCurlyBracket,
        Token::Function,
        Token::Identifier("g".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftCurlyBracket,
        Token::Integer(42),
        Token::RightCurlyBracket,
    ]
)]
#[case::float_literal("12.3", &[ Token::Real(12.3)])]
#[case::zero_integer("0", &[ Token::Integer(0)])]
fn test_code_lexing(#[case] source_code: &str, #[case] expected: &[Token]) {
    let lexer = Token::lexer(source_code);
    let tokens: Vec<Token> = lexer.collect();

    assert_eq!(
        tokens.as_slice(),
        expected,
        "got {:?} instead of {:?}",
        tokens.as_slice(),
        expected
    );
}
