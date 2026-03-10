// SQLRustGo Parser Module
pub use sqlrustgo_common::{SqlError, SqlResult};

pub mod lexer;
pub mod parser;
pub mod token;

pub use lexer::Lexer;
pub use parser::Parser;
pub use token::Token;

pub use parser::parse;
pub use parser::Statement;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        let _lexer = Lexer::new("");
        let tokens = lexer::tokenize("SELECT 1");
        let _parser = Parser::new(tokens);
        let _token = Token::Select;
        let _stmt = Statement::Select(crate::parser::SelectStatement {
            table: "users".to_string(),
            columns: vec![],
            where_clause: None,
            join_clause: None,
            aggregates: vec![],
        });
    }

    #[test]
    fn test_parse_function() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lexer_function() {
        let tokens = lexer::tokenize("SELECT 1");
        assert!(!tokens.is_empty());
    }
}
