use std::{
    fs, io,
    path::{Path, PathBuf},
};

use annotate_snippets::{Level, Message, Renderer, Snippet};
use libbubble::{
    ast::location_to_span,
    parser::ParserError,
    type_system::{BinderError, TypeCheckerError},
};
use thiserror::Error;

pub type CompilerResult<T> = Result<T, CompilerError>;

const RENDERER: Renderer = Renderer::styled();

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Parser error: {error:?}")]
    Parser {
        error: ParserError,
        source_file: PathBuf,
    },
    #[error("Binding error: {error:?}")]
    Binder {
        error: BinderError,
        source_file: PathBuf,
    },
    #[error("Type checking error: {error:?}")]
    TypeChecker {
        error: TypeCheckerError,
        source_file: PathBuf,
    },
    #[error("IO error: {0:?}")]
    IOError(io::Error),
    #[error("Linker error: {0}")]
    Linker(String),
}

fn display_error(msg: Message<'_>) {
    anstream::eprintln!("{}", RENDERER.render(msg));
}

fn emit_parser_error<'a>(error: &'a ParserError, source_file: &'a Path, source_code: &'a str) {
    match error {
        lalrpop_util::ParseError::InvalidToken { location } => display_error(
            Level::Error.title("Invalid token").snippet(
                Snippet::source(source_code).fold(true).annotation(
                    Level::Error
                        .span(*location..*location)
                        .label("Syntax error here"),
                ),
            ),
        ),
        lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => display_error(
            Level::Error
                .title("Unexpected end of file")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(*location..*location)
                                .label("Syntax error here"),
                        ),
                )
                .footers(expected.iter().map(|e| Level::Help.title(e))),
        ),
        lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
            let (begin, _, end) = token;
            display_error(
                Level::Error
                    .title("Unrecognized expression")
                    .snippet(
                        Snippet::source(source_code)
                            .origin(source_file.to_str().expect("failed to decode path"))
                            .fold(true)
                            .annotation(Level::Error.span(*begin..*end)),
                    )
                    .footers(expected.iter().map(|s| Level::Help.title(s))),
            )
        }
        lalrpop_util::ParseError::ExtraToken { token } => {
            let (begin, _, end) = token;
            display_error(
                Level::Error.title("Unrecognized expression").snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(Level::Error.span(*begin..*end)),
                ),
            )
        }
        lalrpop_util::ParseError::User { .. } => display_error(Level::Error.title("Lexical error")),
    }
}

fn emit_binder_error<'a>(error: &'a BinderError, source_code: &'a str, source_file: &'a Path) {
    match error {
        BinderError::UndeclaredVariable { location, name } => display_error(
            Level::Error.title("Undeclared variable").snippet(
                Snippet::source(source_code)
                    .origin(source_file.to_str().expect("failed to decode path"))
                    .fold(true)
                    .annotation(Level::Error.span(location_to_span!(location)).label(name)),
            ),
        ),
        BinderError::UndeclaredStruct { location, name } => display_error(
            Level::Error.title("Undeclared struct").snippet(
                Snippet::source(source_code)
                    .origin(source_file.to_str().expect("failed to decode path"))
                    .fold(true)
                    .annotation(Level::Error.span(location_to_span!(location)).label(name)),
            ),
        ),
        BinderError::UndeclaredFunction { location, name } => display_error(
            Level::Error.title("Undeclared function").snippet(
                Snippet::source(source_code)
                    .origin(source_file.to_str().expect("failed to decode path"))
                    .fold(true)
                    .annotation(Level::Error.span(location_to_span!(location)).label(name)),
            ),
        ),
        BinderError::BadReturn { location } => display_error(
            Level::Error
                .title("Return outside a function")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label("Remove this `return`"),
                        ),
                )
                .footer(Level::Help.title("`return` is only valid inside a function")),
        ),
        BinderError::BadBreak { location } => display_error(
            Level::Error
                .title("Break outside a loop")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label("Remove this `break`"),
                        ),
                )
                .footer(Level::Help.title("`break` is only valid inside a loop")),
        ),
        BinderError::BadContinue { location } => display_error(
            Level::Error
                .title("Continue outside a loop")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label("Remove this `continue`"),
                        ),
                )
                .footer(Level::Help.title("`continue` is only valid inside a loop")),
        ),
        BinderError::NotSubscriptable { location } => display_error(
            Level::Error
                .title("Expression is not subscriptable")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_file.to_str().expect("failed to decode path"))
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location))),
                )
                .footer(Level::Help.title("You can only subscript arrays and strings")),
        ),
        BinderError::NonIdentifierFieldAccess(location) => display_error(
            Level::Error.title("Accessed field is not a name").snippet(
                Snippet::source(source_code)
                    .origin(source_file.to_str().expect("failed to decode path"))
                    .fold(true)
                    .annotation(
                        Level::Error
                            .span(location_to_span!(location))
                            .label("This expression does not design a name"),
                    ),
            ),
        ),
    }
}

fn emit_type_checker_error<'a>(
    error: &'a TypeCheckerError,
    source_code: &'a str,
    source_file: &'a Path,
) {
    let source_path_str = source_file.to_str().expect("failed to decode path");

    match error {
        TypeCheckerError::BadInit {
            location,
            left,
            right,
        } => display_error(
            Level::Error
                .title("Incompatible init expression")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label("Variable type and expression type are not compatible"),
                        ),
                )
                .footers([Level::Note.title(&format!(
                    "Left hand hand side has a type of {left:?} and right and side {right:?}"
                ))]),
        ),
        TypeCheckerError::NonBoolCondition(location, ty) => display_error(
            Level::Error
                .title("Expected boolean expression in condition")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label(&format!("This condition has a type of {ty:?}")),
                        ),
                ),
        ),
        TypeCheckerError::BadAssigment {
            location,
            left,
            right,
        } => display_error(
            Level::Error
                .title("Incompatible type in assignment")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label(&format!("Right hand side has type: {right:?}")),
                        ),
                )
                .footer(Level::Note.title(&format!(
                    "Left hand side has type: {left:?} and right hand side: {right:?}"
                ))),
        ),
        TypeCheckerError::NotCallable(location, _) => display_error(
            Level::Error
                .title("This expression is not callable")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location))),
                ),
        ),
        TypeCheckerError::BadParameterCount {
            location,
            expected,
            got,
        } => display_error(
            Level::Error
                .title("Incorrect number of parameters provided")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location)).label(
                            &format!("Expected {expected} parameters but {got} have been provided"),
                        )),
                ),
        ),
        TypeCheckerError::BadParameter {
            location,
            name,
            expected_type,
            got,
        } => display_error(
            Level::Error
                .title(&format!("Incorrect parameter supplied for {name}"))
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location)).label(
                            &format!(
                                "Expected a parameter of type {expected_type:?} \
                                         but parameter has type {got:?}"
                            ),
                        )),
                ),
        ),
        TypeCheckerError::IncompatibleOperationType {
            location,
            operator,
            left_ty,
            right_ty,
        } => display_error(
            Level::Error
                .title(&format!("Incompatible operations for {operator:?}"))
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location)).label(
                            &format!(
                                r"Left hand side has type {left_ty:?}
                                but right hand side has type {right_ty:?}"
                            ),
                        )),
                ),
        ),
        TypeCheckerError::ReturnTypeMismatch {
            location,
            got,
            expected,
        } => display_error(
            Level::Error
                .title("Expression type is return statement is not correct")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .annotation(Level::Error.span(location_to_span!(location)).label(
                            &format!("Expected expression of type {expected:?} but got {got:?}"),
                        )),
                ),
        ),
        TypeCheckerError::InferenceError(location) => display_error(
            Level::Error
                .title("Cannot infer type")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location))),
                )
                .footers([
                    Level::Note.title("The type couldn't be infered automatically"),
                    Level::Help.title("Consider adding type hints"),
                ]),
        ),
        TypeCheckerError::DifferentTypeInArrayInitializer {
            location,
            first,
            found,
            position,
        } => {
            display_error(
                Level::Error
                    .title("Not homogenous types in array initializer")
                    .snippet(
                        Snippet::source(source_code)
                            .origin(source_path_str)
                            .fold(true)
                            .annotation(
                                Level::Error
                                    .span(location_to_span!(location))
                                    .label(&format!("Parameter at position {position}")),
                            )
                            .annotation(Level::Note.span(location_to_span!(location)).label(
                                &format!(
                                    "The expression has a type of {first:?} but a \
                                                    type of {found:?} is expected"
                                ),
                            )),
                    ),
            )
        }
        TypeCheckerError::NonSubscriptable { location, ty } => display_error(
            Level::Error
                .title("Expression is not subscriptable")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label(&format!("Type {ty:?} cannot be indexed")),
                        ),
                ),
        ),
        TypeCheckerError::IndexNotInteger { location, got } => display_error(
            Level::Error.title("Array index is not an integer").snippet(
                Snippet::source(source_code)
                    .origin(source_path_str)
                    .fold(true)
                    .annotation(
                        Level::Error
                            .span(location_to_span!(location))
                            .label(&format!("This expression has type {got:?}")),
                    ),
            ),
        ),
        TypeCheckerError::DerefNonPointer(location, ..) => display_error(
            Level::Error
                .title("Cannot dereference a non-pointer type")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(
                            Level::Error
                                .span(location_to_span!(location))
                                .label("{ty:?} is not pointer like"),
                        ),
                ),
        ),
        TypeCheckerError::SelfReferentialStruct(location, ..) => display_error(
            Level::Error
                .title("Struct declaration reference itself")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .annotation(Level::Error.span(location_to_span!(location))),
                ),
        ),
        TypeCheckerError::NoSuchField {
            location,
            field_name,
            struct_name,
        } => display_error(
            Level::Error.title("No such field").snippet(
                Snippet::source(source_code)
                    .origin(source_path_str)
                    .fold(true)
                    .annotation(
                        Level::Error
                            .span(location_to_span!(location))
                            .label(&format!("Struct {struct_name} has no field {field_name}")),
                    ),
            ),
        ),
        TypeCheckerError::NonStructLhsAccess(location) => display_error(
            Level::Error
                .title("Left hand side is not a struct")
                .snippet(
                    Snippet::source(source_code)
                        .origin(source_path_str)
                        .fold(true)
                        .annotation(Level::Error.span(location_to_span!(location))),
                ),
        ),
    }
}

fn emit_io_error(error_msg: &str) -> Message<'_> {
    Level::Error.title(error_msg)
}

fn emit_linker_error<'a>(linker_stderr: &'a str, linker_message: &'a str) -> Message<'a> {
    Level::Error.title("linker failed").footers([
        Level::Error.title(linker_stderr),
        Level::Note.title(linker_message),
    ])
}

pub fn print_error(error: &CompilerError, linker_path: &Path) -> io::Result<()> {
    match error {
        CompilerError::Parser { error, source_file } => {
            let source_code = fs::read_to_string(source_file)?;
            emit_parser_error(error, source_file, &source_code);
        }
        CompilerError::Binder { error, source_file } => {
            let source_code = fs::read_to_string(source_file)?;
            emit_binder_error(error, &source_code, source_file);
        }
        CompilerError::TypeChecker { error, source_file } => {
            let source_code = fs::read_to_string(source_file)?;
            emit_type_checker_error(error, &source_code, source_file);
        }
        CompilerError::IOError(e) => {
            let error_msg = e.to_string();
            anstream::eprintln!("{}", RENDERER.render(emit_io_error(&error_msg)));
        }
        CompilerError::Linker(e) => {
            let linker_message = format!(
                "Linker path: {}",
                linker_path.to_str().expect("failed to decode linker path")
            );

            anstream::eprintln!("{}", RENDERER.render(emit_linker_error(e, &linker_message)));
        }
    }

    Ok(())
}
