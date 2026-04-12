//! Parser error types
//!
//! Unified error types for SQL parsing with structured error reporting.

use thiserror::Error;

/// Parser error types
#[derive(Error, Debug)]
pub enum ParseError {
    /// Unexpected token encountered
    #[error("Unexpected token: expected {expected:?}, got {actual:?}")]
    UnexpectedToken { expected: String, actual: String },

    /// Unexpected end of input
    #[error("Unexpected end of input: {context}")]
    UnexpectedEndOfInput { context: String },

    /// Empty input
    #[error("Empty input")]
    EmptyInput,

    /// Syntax error
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    /// Invalid SQL statement
    #[error("Invalid SQL statement: {0}")]
    InvalidStatement(String),

    /// Unsupported feature
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// Tokenization error
    #[error("Tokenization error: {0}")]
    TokenizationError(String),

    /// Generic parse error with context
    #[error("Parse error at {location}: {message}")]
    WithLocation { location: String, message: String },
}

impl ParseError {
    /// Create an unexpected token error
    pub fn unexpected_token<T: std::fmt::Display>(actual: T) -> Self {
        Self::UnexpectedToken {
            expected: "unknown".to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create an unexpected token error with expected type
    pub fn unexpected_token_expected<T: std::fmt::Display, U: std::fmt::Display>(
        expected: T,
        actual: U,
    ) -> Self {
        Self::UnexpectedToken {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create an unexpected end of input error
    pub fn unexpected_end_of_input(context: &str) -> Self {
        Self::UnexpectedEndOfInput {
            context: context.to_string(),
        }
    }

    /// Create a syntax error
    pub fn syntax_error<T: std::fmt::Display>(msg: T) -> Self {
        Self::SyntaxError(msg.to_string())
    }

    /// Create an invalid statement error
    pub fn invalid_statement<T: std::fmt::Display>(msg: T) -> Self {
        Self::InvalidStatement(msg.to_string())
    }

    /// Create an unsupported feature error
    pub fn unsupported_feature<T: std::fmt::Display>(msg: T) -> Self {
        Self::UnsupportedFeature(msg.to_string())
    }
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unexpected_token() {
        let err = ParseError::unexpected_token("SELECT");
        assert!(err.to_string().contains("Unexpected token"));
    }

    #[test]
    fn test_unexpected_end_of_input() {
        let err = ParseError::unexpected_end_of_input("INSERT statement");
        assert!(err.to_string().contains("Unexpected end of input"));
    }

    #[test]
    fn test_syntax_error() {
        let err = ParseError::syntax_error("Invalid WHERE clause");
        assert!(err.to_string().contains("Syntax error"));
    }

    #[test]
    fn test_invalid_statement() {
        let err = ParseError::invalid_statement("Missing FROM clause");
        assert!(err.to_string().contains("Invalid SQL statement"));
    }

    #[test]
    fn test_unsupported_feature() {
        let err = ParseError::unsupported_feature("STORED PROCEDURES");
        assert!(err.to_string().contains("Unsupported feature"));
    }
}
