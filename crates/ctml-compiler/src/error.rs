use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("unexpected end of input at {line}:{column}; expected {expected}")]
    UnexpectedEof {
        expected: &'static str,
        line: usize,
        column: usize,
    },
    #[error("unexpected character '{character}' at {line}:{column}; expected {expected}")]
    UnexpectedCharacter {
        character: char,
        expected: &'static str,
        line: usize,
        column: usize,
    },
    #[error("expected whitespace before attribute at {line}:{column}")]
    ExpectedWhitespace { line: usize, column: usize },
    #[error("unterminated string starting at {line}:{column}")]
    UnterminatedString { line: usize, column: usize },
    #[error("unsupported escape '\\{character}' at {line}:{column}")]
    InvalidEscape {
        character: char,
        line: usize,
        column: usize,
    },
    #[error("syntax error at {line}:{column}: {message}")]
    Syntax {
        message: String,
        line: usize,
        column: usize,
    },
}
