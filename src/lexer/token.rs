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
    fn test_token_display_operators() {
        assert_eq!(Token::Equal.to_string(), "=");
        assert_eq!(Token::NotEqual.to_string(), "<>");
        assert_eq!(Token::Greater.to_string(), ">");
        assert_eq!(Token::Less.to_string(), "<");
        assert_eq!(Token::GreaterEqual.to_string(), ">=");
        assert_eq!(Token::LessEqual.to_string(), "<=");
        assert_eq!(Token::And.to_string(), "AND");
        assert_eq!(Token::Or.to_string(), "OR");
        assert_eq!(Token::Not.to_string(), "NOT");
        assert_eq!(Token::Plus.to_string(), "+");
        assert_eq!(Token::Minus.to_string(), "-");
        assert_eq!(Token::Asterisk.to_string(), "*");
        assert_eq!(Token::Slash.to_string(), "/");
        assert_eq!(Token::Percent.to_string(), "%");
    }

    #[test]
    fn test_token_display_syntax() {
        assert_eq!(Token::LParen.to_string(), "(");
        assert_eq!(Token::RParen.to_string(), ")");
        assert_eq!(Token::Comma.to_string(), ",");
        assert_eq!(Token::Dot.to_string(), ".");
        assert_eq!(Token::Semicolon.to_string(), ";");
        assert_eq!(Token::Colon.to_string(), ":");
        assert_eq!(Token::SingleQuote.to_string(), "'");
        assert_eq!(Token::Star.to_string(), "*");
        assert_eq!(Token::Eof.to_string(), "EOF");
    }

    #[test]
    fn test_token_display_keywords() {
        assert_eq!(Token::From.to_string(), "FROM");
        assert_eq!(Token::Where.to_string(), "WHERE");
        assert_eq!(Token::Insert.to_string(), "INSERT");
        assert_eq!(Token::Into.to_string(), "INTO");
        assert_eq!(Token::Values.to_string(), "VALUES");
        assert_eq!(Token::Update.to_string(), "UPDATE");
        assert_eq!(Token::Set.to_string(), "SET");
        assert_eq!(Token::Delete.to_string(), "DELETE");
        assert_eq!(Token::Create.to_string(), "CREATE");
        assert_eq!(Token::Table.to_string(), "TABLE");
        assert_eq!(Token::Drop.to_string(), "DROP");
        assert_eq!(Token::Alter.to_string(), "ALTER");
        assert_eq!(Token::Index.to_string(), "INDEX");
    }

    #[test]
    fn test_token_display_data_types() {
        assert_eq!(Token::Text.to_string(), "TEXT");
        assert_eq!(Token::Float.to_string(), "FLOAT");
        assert_eq!(Token::Boolean.to_string(), "BOOLEAN");
        assert_eq!(Token::Blob.to_string(), "BLOB");
        assert_eq!(Token::Null.to_string(), "NULL");
    }

    #[test]
    fn test_token_display_transaction() {
        assert_eq!(Token::Begin.to_string(), "BEGIN");
        assert_eq!(Token::Commit.to_string(), "COMMIT");
        assert_eq!(Token::Rollback.to_string(), "ROLLBACK");
        assert_eq!(Token::Grant.to_string(), "GRANT");
        assert_eq!(Token::Revoke.to_string(), "REVOKE");
    }

    #[test]
    fn test_token_display_other_keywords() {
        assert_eq!(Token::On.to_string(), "ON");
        assert_eq!(Token::Primary.to_string(), "PRIMARY");
        assert_eq!(Token::Key.to_string(), "KEY");
    }

    #[test]
    fn test_token_boolean_literal() {
        assert_eq!(Token::BooleanLiteral(true).to_string(), "true");
        assert_eq!(Token::BooleanLiteral(false).to_string(), "false");
    }

    #[test]
    fn test_token_clone() {
        let token = Token::Identifier("test".to_string());
        let cloned = token.clone();
        assert_eq!(token, cloned);
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
    fn test_is_keyword_various() {
        // Data type keywords
        assert!(is_keyword("INTEGER"));
        assert!(is_keyword("TEXT"));
        assert!(is_keyword("FLOAT"));
        assert!(is_keyword("BOOLEAN"));
        assert!(is_keyword("BLOB"));

        // Boolean keywords
        assert!(is_keyword("TRUE"));
        assert!(is_keyword("FALSE"));

        // Logical operators
        assert!(is_keyword("AND"));
        assert!(is_keyword("OR"));
        assert!(is_keyword("NOT"));

        // Non-keyword
        assert!(!is_keyword("users"));
        assert!(!is_keyword("foo"));
        assert!(!is_keyword("bar123"));
    }

    #[test]
    fn test_token_from_keyword() {
        assert_eq!(from_keyword("SELECT"), Some(Token::Select));
        assert_eq!(from_keyword("INSERT"), Some(Token::Insert));
        assert_eq!(from_keyword("UNKNOWN"), None);
        assert_eq!(from_keyword("select"), Some(Token::Select));
        assert_eq!(from_keyword("TRUE"), Some(Token::BooleanLiteral(true)));
    }

    #[test]
    fn test_from_keyword_various() {
        // DML keywords
        assert_eq!(from_keyword("FROM"), Some(Token::From));
        assert_eq!(from_keyword("WHERE"), Some(Token::Where));
        assert_eq!(from_keyword("UPDATE"), Some(Token::Update));
        assert_eq!(from_keyword("SET"), Some(Token::Set));
        assert_eq!(from_keyword("DELETE"), Some(Token::Delete));

        // DDL keywords
        assert_eq!(from_keyword("CREATE"), Some(Token::Create));
        assert_eq!(from_keyword("TABLE"), Some(Token::Table));
        assert_eq!(from_keyword("DROP"), Some(Token::Drop));
        assert_eq!(from_keyword("ALTER"), Some(Token::Alter));
        assert_eq!(from_keyword("INDEX"), Some(Token::Index));

        // Transaction keywords
        assert_eq!(from_keyword("BEGIN"), Some(Token::Begin));
        assert_eq!(from_keyword("COMMIT"), Some(Token::Commit));
        assert_eq!(from_keyword("ROLLBACK"), Some(Token::Rollback));

        // Data types
        assert_eq!(from_keyword("INTEGER"), Some(Token::Integer));
        assert_eq!(from_keyword("TEXT"), Some(Token::Text));
        assert_eq!(from_keyword("FLOAT"), Some(Token::Float));
        assert_eq!(from_keyword("BOOLEAN"), Some(Token::Boolean));
        assert_eq!(from_keyword("BLOB"), Some(Token::Blob));
        assert_eq!(from_keyword("NULL"), Some(Token::Null));

        // Boolean literals
        assert_eq!(from_keyword("FALSE"), Some(Token::BooleanLiteral(false)));

        // Logical operators
        assert_eq!(from_keyword("AND"), Some(Token::And));
        assert_eq!(from_keyword("OR"), Some(Token::Or));
        assert_eq!(from_keyword("NOT"), Some(Token::Not));

        // Case insensitive
        assert_eq!(from_keyword("Select"), Some(Token::Select));
        assert_eq!(from_keyword("FROM"), Some(Token::From));

        // Non-keywords return None
        assert_eq!(from_keyword("users"), None);
        assert_eq!(from_keyword("foo"), None);
    }

    #[test]
    fn test_token_debug() {
        let token = Token::Identifier("test".to_string());
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Identifier"));
    }
}
