//! Compiler errors with source positions.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl Error {
    pub fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            message: message.into(),
            line,
            col,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
