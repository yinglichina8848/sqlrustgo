//! SQLRustGo Database System Library
//! 
//! A Rust implementation of a SQL-92 compliant database system.

pub mod types;
pub mod lexer;
pub mod parser;

pub use types::{Value, SqlError, SqlResult, parse_sql_literal};
pub use lexer::{Token, Lexer, tokenize};
pub use parser::{Statement, parse};

/// Initialize the database system
pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_types_module() {
        let v = types::Value::Integer(42);
        assert_eq!(v.to_string(), "42");
    }

    #[test]
    fn test_lexer() {
        let tokens = tokenize("SELECT id FROM users");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0], Token::Select);
    }

    #[test]
    fn test_parser() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
    }
}
