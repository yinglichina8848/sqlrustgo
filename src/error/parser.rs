//! Parser Error Module

use thiserror::Error;

/// Parser error types
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax error: {message}")]
    SyntaxError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    #[error("Unexpected token: {token}")]
    UnexpectedToken {
        token: String,
        expected: Option<String>,
    },

    #[error("Tokenization error: {0}")]
    TokenizationError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

impl ParserError {
    pub fn new(message: &str) -> Self {
        ParserError::SyntaxError {
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    pub fn with_location(message: &str, line: usize, column: usize) -> Self {
        ParserError::SyntaxError {
            message: message.to_string(),
            line: Some(line),
            column: Some(column),
        }
    }
}
