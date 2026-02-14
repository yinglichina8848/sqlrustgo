//! SQLRustGo Database System Library
//! 
//! A Rust implementation of a SQL-92 compliant database system.

pub mod types;
pub mod lexer;
pub mod parser;
pub mod storage;

pub use types::{Value, SqlError, SqlResult, parse_sql_literal};
pub use lexer::{Token, Lexer, tokenize};
pub use parser::{Statement, parse};
pub use storage::{Page, BufferPool, BPlusTree};

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
    fn test_types() {
        let v = types::Value::Integer(42);
        assert_eq!(v.to_string(), "42");
    }

    #[test]
    fn test_lexer() {
        let tokens = tokenize("SELECT id FROM users");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parser() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage() {
        let page = storage::Page::new(1);
        assert_eq!(page.page_id(), 1);
    }

    #[test]
    fn test_bplus_tree() {
        let mut tree = storage::BPlusTree::new();
        tree.insert(10, 100);
        assert_eq!(tree.search(10), Some(100));
    }
}
