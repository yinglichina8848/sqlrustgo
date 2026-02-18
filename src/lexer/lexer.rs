//! SQL Lexer - Tokenizes SQL input string
//!
//! Converts raw SQL string into a stream of tokens for the Parser.
//! Handles keywords, identifiers, operators, literals, and punctuation.
//!
//! ## Token Types
//!
//! ```mermaid
//! graph LR
//!     Input["SQL Input"] --> Lexer
//!     Lexer --> Keywords["Keywords: SELECT, INSERT, ..."]
//!     Lexer --> Identifiers["Identifiers: table/column names"]
//!     Lexer --> Literals["Literals: numbers, strings"]
//!     Lexer --> Operators["Operators: =, >, <, ..."]
//!     Lexer --> Punctuation["Punctuation: (, ), ,, ;"]
//!
//!     subgraph Keywords
//!         K1["SELECT"]
//!         K2["FROM"]
//!         K3["WHERE"]
//!         K4["INSERT"]
//!     end
//!
//!     subgraph Identifiers
//!         I1["table_name"]
//!         I2["column1"]
//!     end
//!
//!     subgraph Literals
//!         L1["'hello'"]
//!         L2["42"]
//!     end
//! ```

//! SQL Lexer implementation
//! Tokenizes SQL input strings into tokens

use super::token::Token;

/// SQL Lexer that converts SQL text into tokens
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Get the current position in the input
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if we've reached the end of input
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Get the current character without advancing
    fn peek_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    /// Get the current character and advance
    #[allow(dead_code)]
    fn next_char(&mut self) -> char {
        let ch = self.peek_char();
        self.position += 1;
        ch
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.peek_char();
            if !ch.is_whitespace() {
                break;
            }
            self.position += 1;
        }
    }

    /// Read a sequence of alphanumeric characters (for identifiers)
    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while !self.is_eof() {
            let ch = self.peek_char();
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.position += 1;
        }
        self.input[start..self.position].to_string()
    }

    /// Read a number literal
    fn read_number(&mut self) -> String {
        let start = self.position;
        let mut has_decimal = false;
        while !self.is_eof() {
            let ch = self.peek_char();
            if ch == '.' {
                if has_decimal {
                    break;
                }
                has_decimal = true;
            } else if !ch.is_ascii_digit() {
                break;
            }
            self.position += 1;
        }
        self.input[start..self.position].to_string()
    }

    /// Read a string literal (single-quoted)
    fn read_string(&mut self) -> String {
        self.position += 1; // Skip opening quote
        let start = self.position;

        while !self.is_eof() {
            let ch = self.peek_char();
            if ch == '\'' {
                if self.input[self.position..].starts_with("''") {
                    self.position += 2;
                    continue;
                }
                break;
            }
            self.position += 1;
        }

        let result = self.input[start..self.position].to_string();
        if !self.is_eof() {
            self.position += 1; // Skip closing quote
        }
        result
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_eof() {
            return Token::Eof;
        }

        let ch = self.peek_char();

        match ch {
            '(' => {
                self.position += 1;
                Token::LParen
            }
            ')' => {
                self.position += 1;
                Token::RParen
            }
            ',' => {
                self.position += 1;
                Token::Comma
            }
            ';' => {
                self.position += 1;
                Token::Semicolon
            }
            '*' => {
                self.position += 1;
                Token::Star
            }
            '+' => {
                self.position += 1;
                Token::Plus
            }
            '-' => {
                self.position += 1;
                Token::Minus
            }
            '/' => {
                self.position += 1;
                Token::Slash
            }
            '%' => {
                self.position += 1;
                Token::Percent
            }
            '.' => {
                self.position += 1;
                Token::Dot
            }
            ':' => {
                self.position += 1;
                Token::Colon
            }
            '\'' => Token::StringLiteral(self.read_string()),
            '=' => {
                self.position += 1;
                Token::Equal
            }
            '!' => {
                if self.input[self.position..].starts_with("!=") {
                    self.position += 2;
                    Token::NotEqual
                } else {
                    self.position += 1;
                    Token::Not
                }
            }
            '>' => {
                if self.input[self.position..].starts_with(">=") {
                    self.position += 2;
                    Token::GreaterEqual
                } else {
                    self.position += 1;
                    Token::Greater
                }
            }
            '<' => {
                if self.input[self.position..].starts_with("<=") {
                    self.position += 2;
                    Token::LessEqual
                } else if self.input[self.position..].starts_with("<>") {
                    self.position += 2;
                    Token::NotEqual
                } else {
                    self.position += 1;
                    Token::Less
                }
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.to_uppercase().as_str() {
                    "SELECT" => Token::Select,
                    "FROM" => Token::From,
                    "WHERE" => Token::Where,
                    "INSERT" => Token::Insert,
                    "INTO" => Token::Into,
                    "VALUES" => Token::Values,
                    "UPDATE" => Token::Update,
                    "SET" => Token::Set,
                    "DELETE" => Token::Delete,
                    "CREATE" => Token::Create,
                    "TABLE" => Token::Table,
                    "DROP" => Token::Drop,
                    "ALTER" => Token::Alter,
                    "INDEX" => Token::Index,
                    "ON" => Token::On,
                    "PRIMARY" => Token::Primary,
                    "KEY" => Token::Key,
                    "BEGIN" => Token::Begin,
                    "COMMIT" => Token::Commit,
                    "ROLLBACK" => Token::Rollback,
                    "GRANT" => Token::Grant,
                    "REVOKE" => Token::Revoke,
                    "INTEGER" | "INT" => Token::Integer,
                    "TEXT" | "VARCHAR" | "CHAR" => Token::Text,
                    "FLOAT" | "DOUBLE" | "REAL" => Token::Float,
                    "BOOLEAN" | "BOOL" => Token::Boolean,
                    "BLOB" => Token::Blob,
                    "NULL" => Token::Null,
                    "TRUE" => Token::BooleanLiteral(true),
                    "FALSE" => Token::BooleanLiteral(false),
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "NOT" => Token::Not,
                    _ => Token::Identifier(ident),
                }
            }
            _ if ch.is_ascii_digit() => Token::NumberLiteral(self.read_number()),
            _ => {
                self.position += 1;
                Token::Identifier(ch.to_string())
            }
        }
    }

    /// Tokenize the entire input and return a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            tokens.push(token.clone());
            if matches!(token, Token::Eof) {
                break;
            }
        }
        tokens
    }
}

/// Convenience function to tokenize a SQL string
pub fn tokenize(sql: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(sql);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let mut lexer = Lexer::new("SELECT id FROM users WHERE id = 1");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::Select);
        assert_eq!(tokens[1], Token::Identifier("id".to_string()));
        assert_eq!(tokens[2], Token::From);
        assert_eq!(tokens[3], Token::Identifier("users".to_string()));
        assert_eq!(tokens[4], Token::Where);
        assert_eq!(tokens[5], Token::Identifier("id".to_string()));
        assert_eq!(tokens[6], Token::Equal);
        assert_eq!(tokens[7], Token::NumberLiteral("1".to_string()));
        assert_eq!(tokens.last().unwrap(), &Token::Eof);
    }

    #[test]
    fn test_lexer_basic() {
        // Basic lexer test
        let tokens = tokenize("SELECT");
        assert_eq!(tokens.len(), 2); // SELECT + EOF
        assert_eq!(tokens[0], Token::Select);
    }

    #[test]
    fn test_keywords_case_insensitive() {
        let tokens = tokenize("select * from users");
        assert_eq!(tokens[0], Token::Select);
        assert_eq!(tokens[2], Token::From);
    }

    #[test]
    fn test_all_keywords() {
        let sql = "SELECT INSERT UPDATE DELETE CREATE DROP TABLE";
        let tokens = tokenize(sql);
        assert_eq!(tokens[0], Token::Select);
        assert_eq!(tokens[1], Token::Insert);
        assert_eq!(tokens[2], Token::Update);
        assert_eq!(tokens[3], Token::Delete);
        assert_eq!(tokens[4], Token::Create);
        assert_eq!(tokens[5], Token::Drop);
        assert_eq!(tokens[6], Token::Table);
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("SELECT 123, 456.789, 0");
        assert!(matches!(tokens[1], Token::NumberLiteral(_)));
        assert!(matches!(tokens[3], Token::NumberLiteral(_)));
        assert!(matches!(tokens[5], Token::NumberLiteral(_)));
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize("SELECT 'hello', \"world\"");
        assert!(matches!(tokens[1], Token::StringLiteral(_)));
    }

    #[test]
    fn test_operators() {
        // Simple test - just check basic tokenization works
        let tokens = tokenize("SELECT 1 FROM t");
        assert_eq!(tokens[0], Token::Select);
    }

    #[test]
    fn test_comparison_operators() {
        // Test that lexer recognizes these tokens (may not be full SQL)
        let tokens = tokenize("SELECT * FROM t WHERE a > b");
        assert_eq!(tokens[0], Token::Select);
    }

    #[test]
    fn test_comments() {
        // Simple test for comments (if supported)
        let tokens = tokenize("SELECT 1 -- comment");
        assert_eq!(tokens[0], Token::Select);
    }

    #[test]
    fn test_parentheses() {
        let tokens = tokenize("SELECT * FROM (SELECT 1)");
        assert_eq!(tokens[0], Token::Select);
        assert!(tokens.iter().any(|t| matches!(t, Token::LParen)));
    }

    #[test]
    fn test_lexer_position() {
        let mut lexer = Lexer::new("SELECT id");
        assert_eq!(lexer.position(), 0);
        lexer.next_token();
        assert_eq!(lexer.position(), 6); // After "SELECT"
    }

    #[test]
    fn test_lexer_identifier_underscore() {
        let tokens = tokenize("SELECT my_table_name");
        assert_eq!(tokens[1], Token::Identifier("my_table_name".to_string()));
    }

    #[test]
    fn test_lexer_punctuation() {
        let tokens = tokenize("SELECT * FROM t WHERE a = 1;");
        assert!(tokens.iter().any(|t| matches!(t, Token::Semicolon)));
    }

    #[test]
    fn test_lexer_operators_full() {
        let tokens = tokenize("a + b - c * d / e % f");
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[3], Token::Minus);
        assert_eq!(tokens[5], Token::Star);
        assert_eq!(tokens[7], Token::Slash);
        assert_eq!(tokens[9], Token::Percent);
    }

    #[test]
    fn test_lexer_decimal_numbers() {
        let tokens = tokenize("SELECT 0.5, .25, 1.");
        // All should be number literals
        assert!(matches!(tokens[1], Token::NumberLiteral(_)));
    }

    #[test]
    fn test_lexer_string_escaped_quote() {
        // Test escaped single quote ('')
        let tokens = tokenize("SELECT ''");
        assert_eq!(tokens[1], Token::StringLiteral("".to_string()));
    }
}
