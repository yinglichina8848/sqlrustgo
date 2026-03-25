// SQLRustGo Parser Module
pub use sqlrustgo_common::{SqlError, SqlResult};

pub mod lexer;
pub mod parser;
pub mod token;

pub use lexer::Lexer;
pub use parser::Parser;
pub use token::Token;

pub use parser::parse;
pub use parser::CreateViewStatement;
pub use parser::Expression;
pub use parser::GrantStatement;
pub use parser::Privilege;
pub use parser::RevokeStatement;
pub use parser::SetOperation;
pub use parser::SetOperationType;
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
            limit: None,
            offset: None,
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

    #[test]
    fn test_parse_union() {
        let result = parse("SELECT id FROM t1 UNION SELECT id FROM t2");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::SetOperation(_) => {}
            _ => panic!("Expected SetOperation"),
        }
    }

    #[test]
    fn test_parse_union_all() {
        let result = parse("SELECT id FROM t1 UNION ALL SELECT id FROM t2");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::SetOperation(crate::parser::SetOperation {
                op_type: crate::parser::SetOperationType::UnionAll,
                ..
            }) => {}
            _ => panic!("Expected UnionAll"),
        }
    }

    #[test]
    fn test_parse_create_view() {
        let result = parse("CREATE VIEW v AS SELECT id, name FROM users");
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateView(view) => {
                assert_eq!(view.name, "v");
                assert!(view.query.contains("SELECT"));
            }
            _ => panic!("Expected CreateView"),
        }
    }
}

#[test]
fn test_sql92_features() {
    let tests = vec![
        ("SELECT * FROM users LIMIT 10", true),
        ("SELECT * FROM users LIMIT 10 OFFSET 20", true),
        ("INSERT INTO users SET name='test'", true),
        ("ALTER TABLE users ADD COLUMN age INTEGER", true),
        ("ALTER TABLE users DROP COLUMN age", true),
        ("CREATE INDEX idx ON users (name)", true),
        ("CREATE UNIQUE INDEX idx ON users (email)", true),
        ("CREATE TABLE t (id DECIMAL(10,2))", true),
        ("CREATE TABLE t (data JSON)", true),
    ];
    for (sql, should_pass) in tests {
        let result = parse(sql);
        if should_pass {
            assert!(result.is_ok(), "Expected {} to parse OK", sql);
        }
    }
}
