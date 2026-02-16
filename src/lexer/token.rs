//! Token definitions for SQL lexer
//! SQL token types and token struct

use std::fmt;

/// SQL Token types
///
/// ## Categories
///
/// - **Keywords**: SQL reserved words (SELECT, FROM, WHERE, etc.)
/// - **Identifiers**: Table/column names
/// - **Literals**: String, numeric, boolean values
/// - **Operators**: Comparison and arithmetic operators
/// - **Punctuation**: Delimiters and separators
///
/// ## Token Variants
///
/// ### Keywords
/// Data manipulation: `SELECT`, `INSERT`, `UPDATE`, `DELETE`
/// Data definition: `CREATE`, `DROP`, `ALTER`, `TABLE`
/// Transaction control: `BEGIN`, `COMMIT`, `ROLLBACK`
/// Other: `FROM`, `WHERE`, `INTO`, `VALUES`, `SET`, `ON`, `PRIMARY`, `KEY`
///
/// ### Data Types
/// `INTEGER`, `TEXT`, `FLOAT`, `BOOLEAN`, `BLOB`
///
/// ### Operators
/// Comparison: `=`, `!=`, `>`, `<`, `>=`, `<=`
/// Logical: `AND`, `OR`, `NOT`
/// Arithmetic: `+`, `-`, `*`, `/`, `%`
///
/// ### Punctuation
/// `(`, `)`, `,`, `.`, `;`, `:`
///
/// ### Literals
/// `Identifier(String)`, `StringLiteral(String)`, `NumberLiteral(String)`, `BooleanLiteral(bool)`
///
/// ### Special
/// `Eof` - End of file marker

/// Token type enumeration representing all SQL lexical elements
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Drop,
    Alter,
    Index,
    On,
    Primary,
    Key,
    Begin,
    Commit,
    Rollback,
    Grant,
    Revoke,

    // Data Types
    Integer,
    Text,
    Float,
    Boolean,
    Blob,
    Null,

    // Operators
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,

    // Syntax
    LParen,
    RParen,
    Comma,
    Dot,
    Semicolon,
    Colon,
    SingleQuote,

    // Wildcard
    Star, // * for SELECT *

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),

    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "IDENTIFIER({})", s),
            Token::StringLiteral(s) => write!(f, "'{}'", s),
            Token::NumberLiteral(s) => write!(f, "{}", s),
            Token::BooleanLiteral(b) => write!(f, "{}", b),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Check if a string is a SQL keyword
pub fn is_keyword(s: &str) -> bool {
    matches!(
        s.to_uppercase().as_str(),
        "SELECT"
            | "FROM"
            | "WHERE"
            | "INSERT"
            | "INTO"
            | "VALUES"
            | "UPDATE"
            | "SET"
            | "DELETE"
            | "CREATE"
            | "TABLE"
            | "DROP"
            | "ALTER"
            | "INDEX"
            | "ON"
            | "PRIMARY"
            | "KEY"
            | "BEGIN"
            | "COMMIT"
            | "ROLLBACK"
            | "GRANT"
            | "REVOKE"
            | "INTEGER"
            | "TEXT"
            | "FLOAT"
            | "BOOLEAN"
            | "BLOB"
            | "NULL"
            | "TRUE"
            | "FALSE"
            | "AND"
            | "OR"
            | "NOT"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(Token::Select.to_string(), "Select");
        assert_eq!(
            Token::Identifier("users".to_string()).to_string(),
            "IDENTIFIER(users)"
        );
        assert_eq!(
            Token::StringLiteral("hello".to_string()).to_string(),
            "'hello'"
        );
        assert_eq!(Token::NumberLiteral("42".to_string()).to_string(), "42");
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("SELECT"));
        assert!(is_keyword("select"));
        assert!(!is_keyword("users"));
        assert!(is_keyword("TRUE"));
        assert!(is_keyword("null"));
    }
}
