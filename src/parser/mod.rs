//! SQL Parser Module
//! 
//! This module provides SQL parsing functionality.
//! It converts tokens from the lexer into an AST.

use crate::lexer::{Token, Lexer};

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
    pub values: Vec<Expression>,
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
#[derive(Debug, Clone, PartialEq)]
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

impl<> Parser<> {
    /// Create a parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
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
                        columns.push(SelectColumn {
                            name,
                            alias: None,
                        });
                    }
                }
                Some(Token::Comma) => { self.next(); }
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
        
        Ok(Statement::Select(SelectStatement {
            columns,
            table,
            where_clause: None,
        }))
    }
    
    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect(Token::Insert)?;
        self.expect(Token::Into)?;
        
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        
        Ok(Statement::Insert(InsertStatement {
            table,
            columns: Vec::new(),
            values: Vec::new(),
        }))
    }
    
    fn parse_update(&mut self) -> Result<Statement, String> {
        self.expect(Token::Update)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        
        Ok(Statement::Update(UpdateStatement {
            table,
            set_clauses: Vec::new(),
            where_clause: None,
        }))
    }
    
    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.expect(Token::Delete)?;
        self.expect(Token::From)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        
        Ok(Statement::Delete(DeleteStatement {
            table,
            where_clause: None,
        }))
    }
    
    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Create)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        
        Ok(Statement::CreateTable(CreateTableStatement {
            name,
            columns: Vec::new(),
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
        let result = parse("INSERT INTO users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
            }
            _ => panic!("Expected INSERT statement"),
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
