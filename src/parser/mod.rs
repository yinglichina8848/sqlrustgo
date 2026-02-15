//! SQL Parser Module
//!
//! This module provides SQL parsing functionality.
//! It converts tokens from the lexer into an AST.

use crate::lexer::{Lexer, Token};
use serde::{Deserialize, Serialize};

/// SQL Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
}

/// SELECT statement
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub table: String,
    pub where_clause: Option<Expression>,
}

/// Column in SELECT
#[derive(Debug, Clone, PartialEq)]
pub struct SelectColumn {
    pub name: String,
    pub alias: Option<String>,
}

/// INSERT statement
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>, // Multiple rows
}

/// UPDATE statement  
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub table: String,
    pub set_clauses: Vec<(String, Expression)>,
    pub where_clause: Option<Expression>,
}

/// DELETE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expression>,
}

/// CREATE TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

/// DROP TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropTableStatement {
    pub name: String,
}

/// Column definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
}

/// SQL Parser
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Get current token
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Check if at end
    #[allow(dead_code)]
    fn is_eof(&self) -> bool {
        self.position >= self.tokens.len() || matches!(self.current(), Some(Token::Eof))
    }

    /// Advance to next token
    fn next(&mut self) -> Option<Token> {
        self.position += 1;
        self.tokens.get(self.position - 1).cloned()
    }

    /// Expect a specific token
    fn expect(&mut self, expected: Token) -> Result<Token, String> {
        match self.current() {
            Some(t) if t == &expected => Ok(self.next().unwrap()),
            Some(t) => Err(format!("Expected {:?}, got {:?}", expected, t)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    /// Parse a complete SQL statement
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Select) => self.parse_select(),
            Some(Token::Insert) => self.parse_insert(),
            Some(Token::Update) => self.parse_update(),
            Some(Token::Delete) => self.parse_delete(),
            Some(Token::Create) => self.parse_create_table(),
            Some(Token::Drop) => self.parse_drop_table(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.expect(Token::Select)?;

        let mut columns = Vec::new();
        loop {
            match self.current() {
                Some(Token::From) => break,
                Some(Token::Star) => {
                    columns.push(SelectColumn {
                        name: "*".to_string(),
                        alias: None,
                    });
                    self.next();
                }
                Some(Token::Identifier(_)) => {
                    if let Some(Token::Identifier(name)) = self.next() {
                        columns.push(SelectColumn { name, alias: None });
                    }
                }
                Some(Token::Comma) => {
                    self.next();
                }
                _ => {
                    return Err("Expected FROM or column name".to_string());
                }
            }
        }

        self.expect(Token::From)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected table name, got {:?}", t)),
            None => return Err("Expected table name".to_string()),
        };

        // Parse WHERE clause (optional)
        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next(); // consume WHERE
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Select(SelectStatement {
            columns,
            table,
            where_clause,
        }))
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect(Token::Insert)?;
        self.expect(Token::Into)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Check for column list: (col1, col2, ...)
        let columns = if matches!(self.current(), Some(Token::LParen)) {
            self.next(); // consume '('
            let mut cols = Vec::new();
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        cols.push(name.clone());
                        self.next();
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => return Err("Expected column name".to_string()),
                }
            }
            cols
        } else {
            Vec::new()
        };

        // Expect VALUES keyword
        if !matches!(self.current(), Some(Token::Values)) {
            return Err("Expected VALUES".to_string());
        }
        self.next(); // consume VALUES

        // Parse multiple rows: (val1, val2, ...), (val1, val2, ...), ...
        let mut values = Vec::new();

        // Parse first row
        if !matches!(self.current(), Some(Token::LParen)) {
            return Err("Expected ( after VALUES".to_string());
        }

        // Parse all rows
        loop {
            if !matches!(self.current(), Some(Token::LParen)) {
                break;
            }

            // Parse one row
            self.next(); // consume '('
            let mut row = Vec::new();
            loop {
                match self.current() {
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Identifier(name)) => {
                        row.push(Expression::Identifier(name.clone()));
                        self.next();
                    }
                    Some(Token::NumberLiteral(n)) => {
                        row.push(Expression::Literal(n.clone()));
                        self.next();
                    }
                    Some(Token::StringLiteral(s)) => {
                        row.push(Expression::Literal(format!("'{}'", s)));
                        self.next();
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    Some(Token::Null) => {
                        row.push(Expression::Literal("NULL".to_string()));
                        self.next();
                    }
                    Some(Token::Minus) => {
                        // Negative number
                        self.next();
                        if let Some(Token::NumberLiteral(n)) = self.current() {
                            row.push(Expression::Literal(format!("-{}", n)));
                            self.next();
                        } else {
                            return Err("Expected number after -".to_string());
                        }
                    }
                    _ => return Err("Expected value".to_string()),
                }
            }
            values.push(row);

            // Check for more rows: either comma or end
            match self.current() {
                Some(Token::Comma) => {
                    self.next(); // consume comma, continue to next row
                }
                _ => break,
            }
        }

        if values.is_empty() {
            return Err("Expected at least one row of values".to_string());
        }

        Ok(Statement::Insert(InsertStatement {
            table,
            columns,
            values,
        }))
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        self.expect(Token::Update)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Expect SET keyword
        if !matches!(self.current(), Some(Token::Set)) {
            return Err("Expected SET".to_string());
        }
        self.next(); // consume SET

        // Parse SET clause: column = value [, column = value ...]
        let mut set_clauses = Vec::new();
        loop {
            let column = match self.current() {
                Some(Token::Identifier(name)) => name.clone(),
                _ => return Err("Expected column name in SET".to_string()),
            };
            self.next();

            // Expect =
            match self.current() {
                Some(Token::Equal) => {}
                _ => return Err("Expected = in SET clause".to_string()),
            }
            self.next(); // consume =

            // Parse value
            let value = match self.current() {
                Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
                Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
                Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
                Some(Token::Null) => Expression::Literal("NULL".to_string()),
                Some(Token::Minus) => {
                    self.next();
                    if let Some(Token::NumberLiteral(n)) = self.current() {
                        Expression::Literal(format!("-{}", n))
                    } else {
                        return Err("Expected number after -".to_string());
                    }
                }
                _ => return Err("Expected value in SET clause".to_string()),
            };
            self.next();

            set_clauses.push((column, value));

            // Check for more SET clauses or WHERE
            match self.current() {
                Some(Token::Comma) => {
                    self.next(); // consume comma, continue to parse next column
                }
                Some(Token::Where) | None | Some(Token::Eof) => break,
                _ => return Err("Expected , or WHERE".to_string()),
            }
        }

        // Parse WHERE clause (optional)
        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next(); // consume WHERE
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Update(UpdateStatement {
            table,
            set_clauses,
            where_clause,
        }))
    }

    /// Parse a simple expression (for WHERE clause)
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let left = match self.current() {
            Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
            Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
            Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
            Some(Token::Null) => Expression::Literal("NULL".to_string()),
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    Expression::Literal(format!("-{}", n))
                } else {
                    return Err("Expected number after -".to_string());
                }
            }
            _ => return Err("Expected expression".to_string()),
        };
        self.next();

        // Check for binary operator
        let op = match self.current() {
            Some(Token::Equal) => "=",
            Some(Token::NotEqual) => "!=",
            Some(Token::Greater) => ">",
            Some(Token::Less) => "<",
            Some(Token::GreaterEqual) => ">=",
            Some(Token::LessEqual) => "<=",
            _ => return Ok(left), // No operator, return simple expression
        };
        self.next(); // consume operator

        let right = match self.current() {
            Some(Token::Identifier(name)) => Expression::Identifier(name.clone()),
            Some(Token::NumberLiteral(n)) => Expression::Literal(n.clone()),
            Some(Token::StringLiteral(s)) => Expression::Literal(format!("'{}'", s)),
            Some(Token::Null) => Expression::Literal("NULL".to_string()),
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    Expression::Literal(format!("-{}", n))
                } else {
                    return Err("Expected number after -".to_string());
                }
            }
            _ => return Err("Expected value in expression".to_string()),
        };
        self.next();

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.expect(Token::Delete)?;
        self.expect(Token::From)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Parse WHERE clause (optional)
        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next(); // consume WHERE
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Delete(DeleteStatement {
            table,
            where_clause,
        }))
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Create)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Parse optional column definitions: (col1 type, col2 type, ...)
        let mut columns = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next(); // consume '('
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        let col_name = name.clone();
                        self.next();
                        // Parse data type
                        let data_type = match self.current() {
                            Some(Token::Identifier(type_name)) => {
                                let t = type_name.to_uppercase();
                                self.next();
                                t
                            }
                            Some(Token::Integer) => {
                                self.next();
                                "INTEGER".to_string()
                            }
                            Some(Token::Text) => {
                                self.next();
                                "TEXT".to_string()
                            }
                            _ => "INTEGER".to_string(), // default
                        };
                        columns.push(ColumnDefinition {
                            name: col_name,
                            data_type,
                            nullable: true,
                        });
                    }
                    Some(Token::RParen) => {
                        self.next();
                        break;
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }

        Ok(Statement::CreateTable(CreateTableStatement {
            name,
            columns,
        }))
    }

    fn parse_drop_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Drop)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        Ok(Statement::DropTable(DropTableStatement { name }))
    }
}

/// Parse a SQL string into statements
pub fn parse(sql: &str) -> Result<Statement, String> {
    let tokens = Lexer::new(sql).tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select() {
        let result = parse("SELECT id FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.columns.len(), 1);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let result = parse("INSERT INTO users VALUES (1)");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.values.len(), 1); // 1 row
                assert_eq!(i.values[0].len(), 1); // 1 value per row
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_insert_with_values() {
        let result = parse("INSERT INTO users VALUES (1, 'Alice')");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.values.len(), 1); // 1 row
                assert_eq!(i.values[0].len(), 2); // 2 values per row
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_insert_with_columns() {
        let result = parse("INSERT INTO users (id, name) VALUES (1, 'Alice')");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.columns, vec!["id", "name"]);
                assert_eq!(i.values.len(), 1); // 1 row
                assert_eq!(i.values[0].len(), 2); // 2 values
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_insert_multi_row() {
        let result = parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.values.len(), 3); // 3 rows
                assert_eq!(i.values[0].len(), 2); // 2 values per row
                assert_eq!(i.values[1].len(), 2);
                assert_eq!(i.values[2].len(), 2);
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_update() {
        let result = parse("UPDATE users SET name = 'Bob'");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Update(u) => {
                assert_eq!(u.table, "users");
                assert_eq!(u.set_clauses.len(), 1);
                assert_eq!(u.set_clauses[0].0, "name");
            }
            _ => panic!("Expected UPDATE statement"),
        }
    }

    #[test]
    fn test_parse_update_with_where() {
        let result = parse("UPDATE users SET name = 'Bob' WHERE id = 1");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Update(u) => {
                assert_eq!(u.table, "users");
                assert_eq!(u.set_clauses.len(), 1);
                assert!(u.where_clause.is_some());
            }
            _ => panic!("Expected UPDATE statement"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let result = parse("DELETE FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
            }
            _ => panic!("Expected DELETE statement"),
        }
    }

    #[test]
    fn test_parse_delete_with_where() {
        let result = parse("DELETE FROM users WHERE id = 1");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
                assert!(d.where_clause.is_some());
            }
            _ => panic!("Expected DELETE statement"),
        }
    }

    #[test]
    fn test_parse_create() {
        let result = parse("CREATE TABLE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "users");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_create_with_columns() {
        let result = parse("CREATE TABLE users (id INTEGER, name TEXT)");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "users");
                assert_eq!(c.columns.len(), 2);
                assert_eq!(c.columns[0].name, "id");
                assert_eq!(c.columns[1].name, "name");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_drop() {
        let result = parse("DROP TABLE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::DropTable(d) => {
                assert_eq!(d.name, "users");
            }
            _ => panic!("Expected DROP TABLE statement"),
        }
    }
}
