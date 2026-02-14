//! SQL Lexer Module
//! 
//! This module provides lexical analysis (tokenization) for SQL statements.

pub mod token;
pub mod lexer;

pub use token::Token;
pub use lexer::Lexer;

/// Tokenize a SQL string into a vector of tokens
pub fn tokenize(sql: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(sql);
    lexer.tokenize()
}
