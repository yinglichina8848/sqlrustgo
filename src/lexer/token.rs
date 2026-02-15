//! Token definitions for SQL lexer
//! SQL token types and token struct

use std::fmt;

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
            // Keywords and data types - uppercase
            Token::Select => write!(f, "SELECT"),
            Token::From => write!(f, "FROM"),
            Token::Where => write!(f, "WHERE"),
            Token::Insert => write!(f, "INSERT"),
            Token::Into => write!(f, "INTO"),
            Token::Values => write!(f, "VALUES"),
            Token::Update => write!(f, "UPDATE"),
            Token::Set => write!(f, "SET"),
            Token::Delete => write!(f, "DELETE"),
            Token::Create => write!(f, "CREATE"),
            Token::Table => write!(f, "TABLE"),
            Token::Drop => write!(f, "DROP"),
            Token::Alter => write!(f, "ALTER"),
            Token::Index => write!(f, "INDEX"),
            Token::On => write!(f, "ON"),
            Token::Primary => write!(f, "PRIMARY"),
            Token::Key => write!(f, "KEY"),
            Token::Begin => write!(f, "BEGIN"),
            Token::Commit => write!(f, "COMMIT"),
            Token::Rollback => write!(f, "ROLLBACK"),
            Token::Grant => write!(f, "GRANT"),
            Token::Revoke => write!(f, "REVOKE"),
            Token::Integer => write!(f, "INTEGER"),
            Token::Text => write!(f, "TEXT"),
            Token::Float => write!(f, "FLOAT"),
            Token::Boolean => write!(f, "BOOLEAN"),
            Token::Blob => write!(f, "BLOB"),
            Token::Null => write!(f, "NULL"),
            // Operators - uppercase
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::Greater => write!(f, ">"),
            Token::Less => write!(f, "<"),
            Token::GreaterEqual => write!(f, ">="),
            Token::LessEqual => write!(f, "<="),
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            // Syntax
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::SingleQuote => write!(f, "'"),
            Token::Star => write!(f, "*"),
            Token::Eof => write!(f, "EOF"),
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

/// Convert a keyword string to its corresponding Token
pub fn from_keyword(s: &str) -> Option<Token> {
    match s.to_uppercase().as_str() {
        "SELECT" => Some(Token::Select),
        "FROM" => Some(Token::From),
        "WHERE" => Some(Token::Where),
        "INSERT" => Some(Token::Insert),
        "INTO" => Some(Token::Into),
        "VALUES" => Some(Token::Values),
        "UPDATE" => Some(Token::Update),
        "SET" => Some(Token::Set),
        "DELETE" => Some(Token::Delete),
        "CREATE" => Some(Token::Create),
        "TABLE" => Some(Token::Table),
        "DROP" => Some(Token::Drop),
        "ALTER" => Some(Token::Alter),
        "INDEX" => Some(Token::Index),
        "ON" => Some(Token::On),
        "PRIMARY" => Some(Token::Primary),
        "KEY" => Some(Token::Key),
        "BEGIN" => Some(Token::Begin),
        "COMMIT" => Some(Token::Commit),
        "ROLLBACK" => Some(Token::Rollback),
        "GRANT" => Some(Token::Grant),
        "REVOKE" => Some(Token::Revoke),
        "INTEGER" => Some(Token::Integer),
        "TEXT" => Some(Token::Text),
        "FLOAT" => Some(Token::Float),
        "BOOLEAN" => Some(Token::Boolean),
        "BLOB" => Some(Token::Blob),
        "NULL" => Some(Token::Null),
        "TRUE" => Some(Token::BooleanLiteral(true)),
        "FALSE" => Some(Token::BooleanLiteral(false)),
        "AND" => Some(Token::And),
        "OR" => Some(Token::Or),
        "NOT" => Some(Token::Not),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(Token::Select.to_string(), "SELECT");
        assert_eq!(Token::Integer.to_string(), "INTEGER");
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

    #[test]
    fn test_token_from_keyword() {
        assert_eq!(from_keyword("SELECT"), Some(Token::Select));
        assert_eq!(from_keyword("INSERT"), Some(Token::Insert));
        assert_eq!(from_keyword("UNKNOWN"), None);
        assert_eq!(from_keyword("select"), Some(Token::Select));
        assert_eq!(from_keyword("TRUE"), Some(Token::BooleanLiteral(true)));
    }
}
