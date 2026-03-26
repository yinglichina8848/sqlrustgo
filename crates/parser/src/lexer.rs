//! SQL Lexer implementation
//! Tokenizes SQL input strings into tokens

use crate::token::Token;

/// SQL Lexer - Tokenizer
///
/// # What (是什么)
/// Lexer 将原始 SQL 字符串分解为 Token 序列，是编译器的第一阶段
///
/// # Why (为什么)
/// Parser 需要结构化的 Token 而不是原始字符串，Lexer 负责这项转换工作
///
/// # How (如何实现)
/// - 逐字符扫描输入
/// - 识别关键字、标识符、字面量、运算符
/// - 跳过空白字符
/// - 使用有限状态机处理不同 token 类型
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

    /// Skip whitespace and comments
    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.peek_char();

            // Skip whitespace
            if ch.is_whitespace() {
                self.position += 1;
                continue;
            }

            // Skip single-line comments: --
            if ch == '-' && self.input[self.position..].starts_with("--") {
                while !self.is_eof() && self.peek_char() != '\n' {
                    self.position += 1;
                }
                continue;
            }

            // Skip block comments: /* ... */
            if ch == '/' && self.input[self.position..].starts_with("/*") {
                self.position += 2; // skip /*
                while !self.is_eof() && !self.input[self.position..].starts_with("*/") {
                    self.position += 1;
                }
                if !self.is_eof() {
                    self.position += 2; // skip */
                }
                continue;
            }

            break;
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
                    "ADD" => Token::Add,
                    "MODIFY" => Token::Modify,
                    "COLUMN" => Token::Column,
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
                    "UNIQUE" => Token::Unique,
                    "INDEX" => Token::Index,
                    "ON" => Token::On,
                    "PRIMARY" => Token::Primary,
                    "KEY" => Token::Key,
                    "DUPLICATE" => Token::Duplicate,
                    "BEGIN" => Token::Begin,
                    "COMMIT" => Token::Commit,
                    "ROLLBACK" => Token::Rollback,
                    "GRANT" => Token::Grant,
                    "REVOKE" => Token::Revoke,
                    "TO" => Token::To,
                    "GROUP" => Token::Group,
                    "BY" => Token::By,
                    "HAVING" => Token::Having,
                    "ORDER" => Token::Order,
                    "ASC" => Token::Asc,
                    "DESC" => Token::Desc,
                    "NULLS" => Token::Nulls,
                    "FIRST" => Token::First,
                    "LAST" => Token::Last,
                    "ALL" => Token::All,
                    "ANALYZE" => Token::Analyze,
                    "EXPLAIN" => Token::Explain,
                    "UNION" => Token::Union,
                    "INTERSECT" => Token::Intersect,
                    "EXCEPT" => Token::Except,
                    "TRIGGER" => Token::Trigger,
                    "BEFORE" => Token::Before,
                    "AFTER" => Token::After,
                    "FOR" => Token::For,
                    "EACH" => Token::Each,
                    "ROW" => Token::Row,
                    "NEW" => Token::New,
                    "OLD" => Token::Old,
                    "REFERENCING" => Token::Referencing,
                    "PROCEDURE" => Token::Procedure,
                    "CALL" => Token::Call,
                    "RETURN" => Token::Return,
                    "IF" => Token::If,
                    "THEN" => Token::Then,
                    "ELSE" => Token::Else,
                    "ELSIF" => Token::Elsif,
                    "END IF" => Token::EndIf,
                    "WHILE" => Token::While,
                    "LOOP" => Token::Loop,
                    "END LOOP" => Token::EndLoop,
                    "LEAVE" => Token::Leave,
                    "ITERATE" => Token::Iterate,
                    "SIGNAL" => Token::Signal,
                    "VIEW" => Token::View,
                    "AS" => Token::As,
                    "ALL" => Token::All,
                    "INTEGER" | "INT" => Token::Integer,
                    "TEXT" | "VARCHAR" | "CHAR" => Token::Text,
                    "FLOAT" | "DOUBLE" | "REAL" => Token::Float,
                    "DECIMAL" | "NUMERIC" => Token::Decimal,
                    "BOOLEAN" | "BOOL" => Token::Boolean,
                    "BLOB" => Token::Blob,
                    "JSON" => Token::Json,
                    "DATE" => Token::Date,
                    "TIMESTAMP" => Token::Timestamp,
                    "LENGTH" => Token::Length,
                    "UPPER" => Token::Upper,
                    "LOWER" => Token::Lower,
                    "SUBSTR" => Token::Substr,
                    "SUBSTRING" => Token::Substring,
                    "TRIM" => Token::Trim,
                    "NOW" => Token::Now,
                    "CURDATE" => Token::Curdate,
                    "CURTIME" => Token::Curtime,
                    "DATE_ADD" => Token::DateAdd,
                    "DATE_FORMAT" => Token::DateFormat,
                    "COUNT" => Token::Count,
                    "SUM" => Token::Sum,
                    "AVG" => Token::Avg,
                    "MIN" => Token::Min,
                    "MAX" => Token::Max,
                    "TRUE" => Token::BooleanLiteral(true),
                    "FALSE" => Token::BooleanLiteral(false),
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "NOT" => Token::Not,
                    "AUTO_INCREMENT" | "AUTOINCREMENT" => Token::AutoIncrement,
                    "REFERENCES" => Token::References,
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
    fn test_operators() {
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
    fn test_lexer_string_literal() {
        let tokens = Lexer::new("'hello world'").tokenize();
        assert!(matches!(&tokens[0], Token::StringLiteral(s) if s == "hello world"));
    }

    #[test]
    fn test_lexer_operators() {
        let tokens = Lexer::new("<> <= >=").tokenize();
        assert!(matches!(tokens[0], Token::NotEqual));
        assert!(matches!(tokens[1], Token::LessEqual));
        assert!(matches!(tokens[2], Token::GreaterEqual));
    }

    #[test]
    fn test_lexer_multiple_statements() {
        let tokens = Lexer::new("SELECT 1; SELECT 2").tokenize();
        // tokens structure: [Select, NumberLiteral(1), Semicolon, Select, NumberLiteral(2), Eof]
        // Semicolon is at index 2
        assert!(matches!(&tokens[2], Token::Semicolon));
    }

    #[test]
    fn test_lexer_position() {
        let mut lexer = Lexer::new("SELECT 1");
        assert_eq!(lexer.position(), 0);
        lexer.next_token();
        assert_eq!(lexer.position(), 6);
    }

    #[test]
    fn test_lexer_peek_char() {
        let lexer = Lexer::new("SELECT");
        assert_eq!(lexer.peek_char(), 'S');
    }

    #[test]
    fn test_lexer_next_char() {
        let mut lexer = Lexer::new("SELECT");
        assert_eq!(lexer.next_char(), 'S');
        assert_eq!(lexer.position(), 1);
    }

    #[test]
    fn test_lexer_is_eof() {
        let mut lexer = Lexer::new("S");
        assert!(!lexer.is_eof());
        lexer.next_token();
        assert!(lexer.is_eof());
    }

    #[test]
    fn test_lexer_skip_whitespace() {
        let mut lexer = Lexer::new("   SELECT");
        lexer.skip_whitespace();
        assert_eq!(lexer.position(), 3);
    }

    #[test]
    fn test_lexer_read_identifier() {
        let mut lexer = Lexer::new("users123");
        let ident = lexer.read_identifier();
        assert_eq!(ident, "users123");
    }

    #[test]
    fn test_lexer_read_number_integer() {
        let mut lexer = Lexer::new("12345");
        let num = lexer.read_number();
        assert_eq!(num, "12345");
    }

    #[test]
    fn test_lexer_read_number_decimal() {
        let mut lexer = Lexer::new("123.45");
        let num = lexer.read_number();
        assert_eq!(num, "123.45");
    }

    #[test]
    fn test_lexer_read_string() {
        let mut lexer = Lexer::new("'test'");
        let s = lexer.read_string();
        assert_eq!(s, "test");
    }

    #[test]
    fn test_lexer_string_escaped_quote() {
        let mut lexer = Lexer::new("'it''s a test'");
        let s = lexer.read_string();
        assert_eq!(s, "it''s a test");
    }

    #[test]
    fn test_lexer_parens() {
        let tokens = Lexer::new("( )").tokenize();
        assert!(matches!(tokens[0], Token::LParen));
        assert!(matches!(tokens[1], Token::RParen));
    }

    #[test]
    fn test_lexer_comma() {
        let tokens = Lexer::new(",").tokenize();
        assert!(matches!(tokens[0], Token::Comma));
    }

    #[test]
    fn test_lexer_semicolon() {
        let tokens = Lexer::new(";").tokenize();
        assert!(matches!(tokens[0], Token::Semicolon));
    }

    #[test]
    fn test_lexer_star() {
        let tokens = Lexer::new("*").tokenize();
        assert!(matches!(tokens[0], Token::Star));
    }

    #[test]
    fn test_lexer_plus() {
        let tokens = Lexer::new("+").tokenize();
        assert!(matches!(tokens[0], Token::Plus));
    }

    #[test]
    fn test_lexer_minus() {
        let tokens = Lexer::new("-").tokenize();
        assert!(matches!(tokens[0], Token::Minus));
    }

    #[test]
    fn test_lexer_slash() {
        let tokens = Lexer::new("/").tokenize();
        assert!(matches!(tokens[0], Token::Slash));
    }

    #[test]
    fn test_lexer_percent() {
        let tokens = Lexer::new("%").tokenize();
        assert!(matches!(tokens[0], Token::Percent));
    }

    #[test]
    fn test_lexer_dot() {
        let tokens = Lexer::new(".").tokenize();
        assert!(matches!(tokens[0], Token::Dot));
    }

    #[test]
    fn test_lexer_colon() {
        let tokens = Lexer::new(":").tokenize();
        assert!(matches!(tokens[0], Token::Colon));
    }

    #[test]
    fn test_lexer_greater() {
        let tokens = Lexer::new(">").tokenize();
        assert!(matches!(tokens[0], Token::Greater));
    }

    #[test]
    fn test_lexer_less() {
        let tokens = Lexer::new("<").tokenize();
        assert!(matches!(tokens[0], Token::Less));
    }

    #[test]
    fn test_lexer_not() {
        let tokens = Lexer::new("!").tokenize();
        assert!(matches!(tokens[0], Token::Not));
    }

    #[test]
    fn test_lexer_not_equal() {
        let tokens = Lexer::new("!=").tokenize();
        assert!(matches!(tokens[0], Token::NotEqual));
    }

    #[test]
    fn test_lexer_less_equal() {
        let tokens = Lexer::new("<=").tokenize();
        assert!(matches!(tokens[0], Token::LessEqual));
    }

    #[test]
    fn test_lexer_greater_equal() {
        let tokens = Lexer::new(">=").tokenize();
        assert!(matches!(tokens[0], Token::GreaterEqual));
    }

    #[test]
    fn test_lexer_not_equal_alt() {
        let tokens = Lexer::new("<>").tokenize();
        assert!(matches!(tokens[0], Token::NotEqual));
    }

    #[test]
    fn test_lexer_boolean_true() {
        let tokens = tokenize("TRUE");
        assert!(matches!(tokens[0], Token::BooleanLiteral(true)));
    }

    #[test]
    fn test_lexer_boolean_false() {
        let tokens = tokenize("FALSE");
        assert!(matches!(tokens[0], Token::BooleanLiteral(false)));
    }

    #[test]
    fn test_lexer_and() {
        let tokens = tokenize("AND");
        assert!(matches!(tokens[0], Token::And));
    }

    #[test]
    fn test_lexer_or() {
        let tokens = tokenize("OR");
        assert!(matches!(tokens[0], Token::Or));
    }

    #[test]
    fn test_lexer_not_keyword() {
        let tokens = tokenize("NOT");
        assert!(matches!(tokens[0], Token::Not));
    }

    #[test]
    fn test_lexer_into() {
        let tokens = tokenize("INTO");
        assert!(matches!(tokens[0], Token::Into));
    }

    #[test]
    fn test_lexer_values() {
        let tokens = tokenize("VALUES");
        assert!(matches!(tokens[0], Token::Values));
    }

    #[test]
    fn test_lexer_set() {
        let tokens = tokenize("SET");
        assert!(matches!(tokens[0], Token::Set));
    }

    #[test]
    fn test_lexer_alter() {
        let tokens = tokenize("ALTER");
        assert!(matches!(tokens[0], Token::Alter));
    }

    #[test]
    fn test_lexer_index() {
        let tokens = tokenize("INDEX");
        assert!(matches!(tokens[0], Token::Index));
    }

    #[test]
    fn test_lexer_on() {
        let tokens = tokenize("ON");
        assert!(matches!(tokens[0], Token::On));
    }

    #[test]
    fn test_lexer_primary() {
        let tokens = tokenize("PRIMARY");
        assert!(matches!(tokens[0], Token::Primary));
    }

    #[test]
    fn test_lexer_key() {
        let tokens = tokenize("KEY");
        assert!(matches!(tokens[0], Token::Key));
    }

    #[test]
    fn test_lexer_begin() {
        let tokens = tokenize("BEGIN");
        assert!(matches!(tokens[0], Token::Begin));
    }

    #[test]
    fn test_lexer_commit() {
        let tokens = tokenize("COMMIT");
        assert!(matches!(tokens[0], Token::Commit));
    }

    #[test]
    fn test_lexer_rollback() {
        let tokens = tokenize("ROLLBACK");
        assert!(matches!(tokens[0], Token::Rollback));
    }

    #[test]
    fn test_lexer_grant() {
        let tokens = tokenize("GRANT");
        assert!(matches!(tokens[0], Token::Grant));
    }

    #[test]
    fn test_lexer_revoke() {
        let tokens = tokenize("REVOKE");
        assert!(matches!(tokens[0], Token::Revoke));
    }

    #[test]
    fn test_lexer_analyze() {
        let tokens = tokenize("ANALYZE");
        assert!(matches!(tokens[0], Token::Analyze));
    }

    #[test]
    fn test_lexer_integer_types() {
        let tokens = tokenize("INTEGER");
        assert!(matches!(tokens[0], Token::Integer));
        let tokens2 = tokenize("INT");
        assert!(matches!(tokens2[0], Token::Integer));
    }

    #[test]
    fn test_lexer_text_types() {
        let tokens = tokenize("TEXT");
        assert!(matches!(tokens[0], Token::Text));
        let tokens2 = tokenize("VARCHAR");
        assert!(matches!(tokens2[0], Token::Text));
        let tokens3 = tokenize("CHAR");
        assert!(matches!(tokens3[0], Token::Text));
    }

    #[test]
    fn test_lexer_float_types() {
        let tokens = tokenize("FLOAT");
        assert!(matches!(tokens[0], Token::Float));
        let tokens2 = tokenize("DOUBLE");
        assert!(matches!(tokens2[0], Token::Float));
        let tokens3 = tokenize("REAL");
        assert!(matches!(tokens3[0], Token::Float));
    }

    #[test]
    fn test_lexer_boolean_types() {
        let tokens = tokenize("BOOLEAN");
        assert!(matches!(tokens[0], Token::Boolean));
        let tokens2 = tokenize("BOOL");
        assert!(matches!(tokens2[0], Token::Boolean));
    }

    #[test]
    fn test_lexer_blob() {
        let tokens = tokenize("BLOB");
        assert!(matches!(tokens[0], Token::Blob));
    }

    #[test]
    fn test_lexer_identifier_with_underscore() {
        let tokens = tokenize("user_name");
        assert!(matches!(&tokens[0], Token::Identifier(s) if s == "user_name"));
    }

    #[test]
    fn test_lexer_mixed_case_keywords() {
        let tokens = tokenize("SeLeCt");
        assert!(matches!(tokens[0], Token::Select));
    }

    #[test]
    fn test_lexer_empty_string() {
        let tokens = Lexer::new("''").tokenize();
        assert!(matches!(&tokens[0], Token::StringLiteral(s) if s.is_empty()));
    }

    #[test]
    fn test_lexer_number_only() {
        let tokens = tokenize("42");
        assert!(matches!(&tokens[0], Token::NumberLiteral(n) if n == "42"));
    }

    #[test]
    fn test_lexer_complex_sql() {
        let sql = "SELECT id, name FROM users WHERE age > 18 AND status = 'active'";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Select));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Comma));
        assert!(matches!(tokens[3], Token::Identifier(_)));
        assert!(matches!(tokens[4], Token::From));
        assert!(matches!(tokens[5], Token::Identifier(_)));
        assert!(matches!(tokens[6], Token::Where));
        assert!(matches!(tokens[7], Token::Identifier(_)));
        assert!(matches!(tokens[8], Token::Greater));
        assert!(matches!(tokens[9], Token::NumberLiteral(_)));
        assert!(matches!(tokens[10], Token::And));
    }

    #[test]
    fn test_lexer_insert_statement() {
        let sql = "INSERT INTO users (name, age) VALUES ('John', 25)";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Insert));
        assert!(matches!(tokens[1], Token::Into));
        assert!(matches!(tokens[2], Token::Identifier(_)));
        assert!(matches!(tokens[3], Token::LParen));
    }

    #[test]
    fn test_lexer_update_statement() {
        let sql = "UPDATE users SET name = 'Jane' WHERE id = 1";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Update));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Set));
    }

    #[test]
    fn test_lexer_delete_statement() {
        let sql = "DELETE FROM users WHERE id = 1";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Delete));
        assert!(matches!(tokens[1], Token::From));
    }

    #[test]
    fn test_lexer_create_table() {
        let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Create));
        assert!(matches!(tokens[1], Token::Table));
    }

    #[test]
    fn test_lexer_drop_table() {
        let sql = "DROP TABLE users";
        let tokens = tokenize(sql);
        assert!(matches!(tokens[0], Token::Drop));
        assert!(matches!(tokens[1], Token::Table));
    }

    #[test]
    fn test_lexer_whitespace_tabs() {
        let tokens = Lexer::new("\t\n\r SELECT").tokenize();
        assert!(matches!(tokens[0], Token::Select));
    }

    #[test]
    fn test_lexer_multiple_spaces() {
        let tokens = Lexer::new("   SELECT   ").tokenize();
        assert!(matches!(tokens[0], Token::Select));
    }

    #[test]
    fn test_lexer_eof_token() {
        let tokens = tokenize("SELECT");
        assert!(matches!(tokens[1], Token::Eof));
    }

    #[test]
    fn test_lexer_unknown_char() {
        let tokens = tokenize("@");
        assert!(matches!(&tokens[0], Token::Identifier(s) if s == "@"));
    }

    #[test]
    fn test_lexer_decimal_number() {
        let tokens = tokenize("3.14");
        assert!(matches!(&tokens[0], Token::NumberLiteral(n) if n == "3.14"));
    }

    #[test]
    fn test_lexer_string_with_escaped_quote() {
        let tokens = tokenize("'it''s'");
        assert!(matches!(&tokens[0], Token::StringLiteral(s) if s == "it''s"));
    }

    #[test]
    fn test_lexer_date_keyword() {
        let tokens = tokenize("DATE");
        assert!(matches!(&tokens[0], Token::Date));
    }

    #[test]
    fn test_lexer_timestamp_keyword() {
        let tokens = tokenize("TIMESTAMP");
        assert!(matches!(&tokens[0], Token::Timestamp));
    }

    #[test]
    fn test_lexer_date_type_in_create() {
        let sql = "CREATE TABLE t (d DATE, ts TIMESTAMP)";
        let tokens = tokenize(sql);
        assert!(matches!(&tokens[5], Token::Date));
        // Find TIMESTAMP position (after comma and ts identifier)
        let ts_pos = tokens.iter().position(|t| matches!(t, Token::Timestamp));
        assert!(
            ts_pos.is_some(),
            "TIMESTAMP not found in tokens: {:?}",
            tokens
        );
    }
}
