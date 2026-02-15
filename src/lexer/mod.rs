//! SQL Lexer Module
//!
//! This module provides lexical analysis (tokenization) for SQL statements.

pub mod lexer;
pub mod token;

pub use lexer::Lexer;
pub use token::Token;

/// Tokenize a SQL string into a vector of tokens
pub fn tokenize(sql: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(sql);
    lexer.tokenize()
}
