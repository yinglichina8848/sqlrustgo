//! SQL Parser Module
//!
//! # What (是什么)
//! Parser 将 Lexer 输出的 Token 序列转换为抽象语法树 (AST)
//!
//! # Why (为什么)
//! Token 序列只是单词的列表，无法表达 SQL 语句的层级结构
//! AST 将单词组织成有意义的树结构，表示查询的语义
//!
//! # How (如何实现)
//! - 递归下降解析器：自顶向下处理 SQL 语句
//! - 每个 Statement 类型有对应的 parse_xxx 方法
//! - 支持：SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE
//! - 表达式解析支持基本二元运算

use crate::lexer::Lexer;
use crate::token::Token;
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
    Analyze(AnalyzeStatement),
    WithSelect(WithSelect),
}

/// Common Table Expression (CTE)
#[derive(Debug, Clone, PartialEq)]
pub struct CommonTableExpression {
    pub name: String,
    pub columns: Vec<String>,
    pub subquery: Box<SelectStatement>,
}

/// WITH clause for CTEs
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub recursive: bool,
    pub ctes: Vec<CommonTableExpression>,
}

/// SELECT with optional WITH clause
#[derive(Debug, Clone, PartialEq)]
pub struct WithSelect {
    pub with_clause: Option<WithClause>,
    pub select: SelectStatement,
}

/// ANALYZE statement for collecting statistics
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzeStatement {
    pub table_name: Option<String>,
}

/// Join type
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// Aggregate function
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// Join clause
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: String,
    pub on_clause: (Expression, Expression),
}

/// Aggregate function call
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateCall {
    pub func: AggregateFunction,
    pub args: Vec<Expression>,
    pub distinct: bool,
}

/// SELECT statement
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub table: String,
    pub where_clause: Option<Expression>,
    pub join_clause: Option<JoinClause>,
    pub aggregates: Vec<AggregateCall>,
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
    pub values: Vec<Vec<Expression>>,         // For INSERT VALUES
    pub select: Option<Box<SelectStatement>>, // For INSERT SELECT
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
    pub constraints: Vec<TableConstraint>,
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
    pub primary_key: bool,
    pub default_value: Option<String>,
    pub references: Option<ForeignKeyRef>,
}

/// Foreign key referential action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReferentialAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}

/// Foreign key reference (for REFERENCES clause)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<ReferentialAction>,
    pub on_update: Option<ReferentialAction>,
}

/// Table-level constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TableConstraint {
    PrimaryKey {
        columns: Vec<String>,
    },
    ForeignKey {
        columns: Vec<String>,
        referenced_table: String,
        referenced_columns: Vec<String>,
        on_delete: Option<ReferentialAction>,
        on_update: Option<ReferentialAction>,
    },
    Unique {
        columns: Vec<String>,
    },
    Check {
        expression: String,
    },
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Subquery(Box<SelectStatement>),
    In(Box<Expression>, Box<SelectStatement>),
    NotIn(Box<Expression>, Box<SelectStatement>),
    Exists(Box<SelectStatement>),
    NotExists(Box<SelectStatement>),
    QuantifiedOp(Box<Expression>, String, Box<SelectStatement>),
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
            Some(Token::Analyze) => self.parse_analyze(),
            Some(Token::With) => self.parse_with_select(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        let select = self.parse_select_statement()?;
        Ok(Statement::Select(select))
    }

    fn parse_with_select(&mut self) -> Result<Statement, String> {
        self.expect(Token::With)?;

        let recursive = if matches!(self.current(), Some(Token::Recursive)) {
            self.next();
            true
        } else {
            false
        };

        let mut ctes = Vec::new();
        loop {
            let cte_name = match self.next() {
                Some(Token::Identifier(name)) => name,
                _ => return Err("Expected CTE name".to_string()),
            };

            let columns = if matches!(self.current(), Some(Token::LParen)) {
                self.next();
                let mut cols = Vec::new();
                loop {
                    match self.current() {
                        Some(Token::Identifier(name)) => {
                            cols.push(name.clone());
                            self.next();
                        }
                        Some(Token::Comma) => {
                            self.next();
                        }
                        Some(Token::RParen) => {
                            self.next();
                            break;
                        }
                        _ => return Err("Expected column name".to_string()),
                    }
                }
                cols
            } else {
                Vec::new()
            };

            self.expect(Token::As)?;
            self.expect(Token::LParen)?;
            let subquery = self.parse_select_statement()?;
            self.expect(Token::RParen)?;

            ctes.push(CommonTableExpression {
                name: cte_name,
                columns,
                subquery: Box::new(subquery),
            });

            if matches!(self.current(), Some(Token::Comma)) {
                self.next();
                continue;
            }
            break;
        }

        let select = self.parse_select_statement()?;

        Ok(Statement::WithSelect(WithSelect {
            with_clause: Some(WithClause { recursive, ctes }),
            select,
        }))
    }

    fn parse_select_statement(&mut self) -> Result<SelectStatement, String> {
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

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(SelectStatement {
            columns,
            table,
            where_clause,
            join_clause: None,
            aggregates: vec![],
        })
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

        // Check if INSERT VALUES or INSERT SELECT
        let (values, select) = if matches!(self.current(), Some(Token::Values)) {
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

                match self.current() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }

            if values.is_empty() {
                return Err("Expected at least one row of values".to_string());
            }

            (values, None)
        } else if matches!(self.current(), Some(Token::Select)) {
            let select_stmt = self.parse_select()?;
            match select_stmt {
                Statement::Select(s) => (Vec::new(), Some(Box::new(s))),
                _ => return Err("Expected SELECT statement".to_string()),
            }
        } else {
            return Err("Expected VALUES or SELECT".to_string());
        };

        Ok(Statement::Insert(InsertStatement {
            table,
            columns,
            values,
            select,
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
    /// Supports: comparison operators (=, !=, >, <, >=, <=)
    /// Logical operators: AND, OR
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or_expression()
    }

    /// Parse OR expression (lowest precedence)
    fn parse_or_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and_expression()?;

        while let Some(Token::Or) = self.current() {
            self.next(); // consume OR
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp(Box::new(left), "OR".to_string(), Box::new(right));
        }

        Ok(left)
    }

    /// Parse AND expression (higher precedence than OR)
    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison_expression()?;

        while let Some(Token::And) = self.current() {
            self.next(); // consume AND
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp(Box::new(left), "AND".to_string(), Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_primary_expression()?;

        if matches!(self.current(), Some(Token::In)) {
            self.next();
            self.expect(Token::LParen)?;
            let subquery = self.parse_select_statement()?;
            self.expect(Token::RParen)?;
            return Ok(Expression::In(Box::new(left), Box::new(subquery)));
        }

        if matches!(self.current(), Some(Token::Not)) {
            self.next();
            if matches!(self.current(), Some(Token::In)) {
                self.next();
                self.expect(Token::LParen)?;
                let subquery = self.parse_select_statement()?;
                self.expect(Token::RParen)?;
                return Ok(Expression::NotIn(Box::new(left), Box::new(subquery)));
            }
            return Err("NOT must be followed by IN or EXISTS".to_string());
        }

        let op = match self.current() {
            Some(Token::Equal) => "=",
            Some(Token::NotEqual) => "!=",
            Some(Token::Greater) => ">",
            Some(Token::Less) => "<",
            Some(Token::GreaterEqual) => ">=",
            Some(Token::LessEqual) => "<=",
            _ => return Ok(left),
        };
        self.next();

        let right = self.parse_primary_expression()?;

        if matches!(
            self.current(),
            Some(Token::All) | Some(Token::Any) | Some(Token::Some)
        ) {
            let quantifier = match self.current() {
                Some(Token::All) => "ALL",
                Some(Token::Any) => "ANY",
                Some(Token::Some) => "SOME",
                _ => {
                    return Ok(Expression::BinaryOp(
                        Box::new(left),
                        op.to_string(),
                        Box::new(right),
                    ))
                }
            };
            self.next();
            self.expect(Token::LParen)?;
            let subquery = self.parse_select_statement()?;
            self.expect(Token::RParen)?;
            Ok(Expression::QuantifiedOp(
                Box::new(Expression::BinaryOp(
                    Box::new(left),
                    op.to_string(),
                    Box::new(right),
                )),
                quantifier.to_string(),
                Box::new(subquery),
            ))
        } else {
            Ok(Expression::BinaryOp(
                Box::new(left),
                op.to_string(),
                Box::new(right),
            ))
        }
    }

    /// Parse primary expression (identifier, literal, or parenthesized)
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.current() {
            Some(Token::Identifier(name)) => {
                let expr = Expression::Identifier(name.clone());
                self.next();
                Ok(expr)
            }
            Some(Token::NumberLiteral(n)) => {
                let expr = Expression::Literal(n.clone());
                self.next();
                Ok(expr)
            }
            Some(Token::StringLiteral(s)) => {
                let expr = Expression::Literal(format!("'{}'", s));
                self.next();
                Ok(expr)
            }
            Some(Token::Null) => {
                let expr = Expression::Literal("NULL".to_string());
                self.next();
                Ok(expr)
            }
            Some(Token::Minus) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    let expr = Expression::Literal(format!("-{}", n));
                    self.next();
                    Ok(expr)
                } else {
                    Err("Expected number after -".to_string())
                }
            }
            Some(Token::LParen) => {
                self.next();
                match self.current() {
                    Some(Token::Select) => {
                        let subquery = self.parse_select_statement()?;
                        self.expect(Token::RParen)?;
                        Ok(Expression::Subquery(Box::new(subquery)))
                    }
                    _ => {
                        let expr = self.parse_or_expression()?;
                        self.expect(Token::RParen)?;
                        Ok(expr)
                    }
                }
            }
            Some(Token::Exists) => {
                self.next();
                self.expect(Token::LParen)?;
                let subquery = self.parse_select_statement()?;
                self.expect(Token::RParen)?;
                Ok(Expression::Exists(Box::new(subquery)))
            }
            Some(Token::Not) => {
                self.next();
                if matches!(self.current(), Some(Token::Exists)) {
                    self.next();
                    self.expect(Token::LParen)?;
                    let subquery = self.parse_select_statement()?;
                    self.expect(Token::RParen)?;
                    Ok(Expression::NotExists(Box::new(subquery)))
                } else {
                    Err("NOT without EXISTS".to_string())
                }
            }
            _ => Err("Expected expression".to_string()),
        }
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

        let mut columns = Vec::new();
        let mut constraints = Vec::new();

        if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            loop {
                match self.current() {
                    Some(Token::Identifier(_)) => {
                        let col_def = self.parse_column_definition()?;
                        columns.push(col_def);
                    }
                    Some(Token::Primary) => {
                        self.next();
                        self.expect(Token::Key)?;
                        let columns = self.parse_column_list()?;
                        constraints.push(TableConstraint::PrimaryKey { columns });
                    }
                    Some(Token::Foreign) => {
                        let fk = self.parse_foreign_key_constraint()?;
                        constraints.push(fk);
                    }
                    Some(Token::Unique) => {
                        self.next();
                        let columns = self.parse_column_list()?;
                        constraints.push(TableConstraint::Unique { columns });
                    }
                    Some(Token::Check) => {
                        self.next();
                        self.expect(Token::LParen)?;
                        let expr = self.parse_expression()?;
                        self.expect(Token::RParen)?;
                        constraints.push(TableConstraint::Check {
                            expression: format!("{:?}", expr),
                        });
                    }
                    Some(Token::Constraint) => {
                        self.next();
                        if let Some(Token::Identifier(_name)) = self.next() {
                            self.next();
                            match self.current() {
                                Some(Token::Primary) => {
                                    self.next();
                                    self.expect(Token::Key)?;
                                    let cols = self.parse_column_list()?;
                                    constraints.push(TableConstraint::PrimaryKey { columns: cols });
                                }
                                Some(Token::Foreign) => {
                                    let fk = self.parse_foreign_key_constraint()?;
                                    constraints.push(fk);
                                }
                                Some(Token::Unique) => {
                                    self.next();
                                    let cols = self.parse_column_list()?;
                                    constraints.push(TableConstraint::Unique { columns: cols });
                                }
                                Some(Token::Check) => {
                                    self.next();
                                    self.expect(Token::LParen)?;
                                    let expr = self.parse_expression()?;
                                    self.expect(Token::RParen)?;
                                    constraints.push(TableConstraint::Check {
                                        expression: format!("{:?}", expr),
                                    });
                                }
                                _ => return Err("Expected constraint type".to_string()),
                            }
                        }
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
            constraints,
        }))
    }

    fn parse_column_definition(&mut self) -> Result<ColumnDefinition, String> {
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected column name".to_string()),
        };

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
            _ => "INTEGER".to_string(),
        };

        let mut nullable = true;
        let mut primary_key = false;
        let mut default_value = None;
        let mut references = None;

        loop {
            match self.current() {
                Some(Token::Not) => {
                    self.next();
                    if let Some(Token::Null) = self.current() {
                        self.next();
                        nullable = false;
                    }
                }
                Some(Token::Null) => {
                    self.next();
                    nullable = true;
                }
                Some(Token::Primary) => {
                    self.next();
                    self.expect(Token::Key)?;
                    primary_key = true;
                    nullable = false;
                }
                Some(Token::Default) => {
                    self.next();
                    default_value = Some(self.parse_simple_value()?);
                }
                Some(Token::References) => {
                    self.next();
                    let ref_table = match self.next() {
                        Some(Token::Identifier(name)) => name,
                        _ => return Err("Expected referenced table name".to_string()),
                    };
                    let ref_columns = self.parse_column_list()?;
                    let (on_delete, on_update) = self.parse_referential_actions()?;
                    references = Some(ForeignKeyRef {
                        columns: vec![name.clone()],
                        referenced_table: ref_table,
                        referenced_columns: ref_columns,
                        on_delete,
                        on_update,
                    });
                }
                _ => break,
            }
        }

        Ok(ColumnDefinition {
            name,
            data_type,
            nullable,
            primary_key,
            default_value,
            references,
        })
    }

    fn parse_foreign_key_constraint(&mut self) -> Result<TableConstraint, String> {
        self.expect(Token::Foreign)?;
        self.expect(Token::Key)?;
        let columns = self.parse_column_list()?;
        self.expect(Token::References)?;
        let referenced_table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected referenced table name".to_string()),
        };
        let referenced_columns = self.parse_column_list()?;
        let (on_delete, on_update) = self.parse_referential_actions()?;
        Ok(TableConstraint::ForeignKey {
            columns,
            referenced_table,
            referenced_columns,
            on_delete,
            on_update,
        })
    }

    fn parse_referential_actions(
        &mut self,
    ) -> Result<(Option<ReferentialAction>, Option<ReferentialAction>), String> {
        let mut on_delete = None;
        let mut on_update = None;
        loop {
            match self.current() {
                Some(Token::On) => {
                    self.next();
                    match self.current() {
                        Some(Token::Delete) => {
                            self.next();
                            on_delete = Some(self.parse_referential_action()?);
                        }
                        Some(Token::Update) => {
                            self.next();
                            on_update = Some(self.parse_referential_action()?);
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        Ok((on_delete, on_update))
    }

    fn parse_referential_action(&mut self) -> Result<ReferentialAction, String> {
        match self.current() {
            Some(Token::Cascade) => {
                self.next();
                Ok(ReferentialAction::Cascade)
            }
            Some(Token::Set) => {
                self.next();
                if let Some(Token::Null) = self.current() {
                    self.next();
                    Ok(ReferentialAction::SetNull)
                } else {
                    Err("Expected NULL after SET".to_string())
                }
            }
            Some(Token::Restrict) => {
                self.next();
                Ok(ReferentialAction::Restrict)
            }
            Some(Token::No) => {
                self.next();
                if let Some(Token::Action) = self.current() {
                    self.next();
                    Ok(ReferentialAction::NoAction)
                } else {
                    Err("Expected ACTION after NO".to_string())
                }
            }
            _ => Err(
                "Expected referential action (CASCADE, SET NULL, RESTRICT, NO ACTION)".to_string(),
            ),
        }
    }

    fn parse_simple_value(&mut self) -> Result<String, String> {
        let token = self.current().cloned();
        match token {
            Some(Token::NumberLiteral(n)) => {
                self.next();
                Ok(n)
            }
            Some(Token::StringLiteral(s)) => {
                self.next();
                Ok(format!("'{}'", s))
            }
            Some(Token::Identifier(name)) => {
                self.next();
                Ok(name)
            }
            Some(Token::Null) => {
                self.next();
                Ok("NULL".to_string())
            }
            _ => Err("Expected a value".to_string()),
        }
    }

    fn parse_column_list(&mut self) -> Result<Vec<String>, String> {
        let mut columns = Vec::new();
        self.expect(Token::LParen)?;
        loop {
            match self.current() {
                Some(Token::Identifier(name)) => {
                    columns.push(name.clone());
                    self.next();
                }
                Some(Token::Comma) => {
                    self.next();
                }
                Some(Token::RParen) => {
                    self.next();
                    break;
                }
                _ => break,
            }
        }
        Ok(columns)
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

    fn parse_analyze(&mut self) -> Result<Statement, String> {
        self.expect(Token::Analyze)?;

        let table_name = match self.current() {
            Some(Token::Identifier(name)) => {
                let n = name.clone();
                self.next();
                Some(n)
            }
            Some(Token::Semicolon) | None => None,
            _ => return Err("Expected table name or semicolon".to_string()),
        };

        Ok(Statement::Analyze(AnalyzeStatement { table_name }))
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
    fn test_parse_insert_select() {
        let result = parse("INSERT INTO users SELECT * FROM old_users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert!(
                    i.values.is_empty(),
                    "INSERT VALUES should be empty for INSERT SELECT"
                );
                assert!(
                    i.select.is_some(),
                    "INSERT SELECT should have a select statement"
                );
                let select = i.select.as_ref().unwrap();
                assert_eq!(select.table, "old_users");
                assert_eq!(select.columns.len(), 1); // * expands to one column
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
    fn test_parse_insert_select_with_columns() {
        let result =
            parse("INSERT INTO users (id, name) SELECT id, name FROM old_users WHERE id > 0");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert_eq!(i.columns, vec!["id".to_string(), "name".to_string()]);
                assert!(
                    i.select.is_some(),
                    "INSERT SELECT should have a select statement"
                );
                let select = i.select.as_ref().unwrap();
                assert_eq!(select.table, "old_users");
                assert!(select.where_clause.is_some());
            }
            _ => panic!("Expected INSERT statement"),
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
    fn test_parse_create_with_foreign_key() {
        let result = parse("CREATE TABLE t (a INTEGER REFERENCES u(b))");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "t");
                assert_eq!(c.columns.len(), 1, "Actual columns: {:?}", c.columns);
                assert!(c.columns[0].references.is_some(), "No FK ref found");
                let fk = c.columns[0].references.as_ref().unwrap();
                assert_eq!(fk.referenced_table, "u");
                assert_eq!(fk.referenced_columns, vec!["b".to_string()]);
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

    #[test]
    fn test_parse_create_with_table_constraint_fk() {
        let result = parse("CREATE TABLE orders (id INTEGER, user_id INTEGER, FOREIGN KEY (user_id) REFERENCES users(id))");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "orders");
                assert_eq!(c.columns.len(), 2);
                assert_eq!(c.constraints.len(), 1);
                match &c.constraints[0] {
                    TableConstraint::ForeignKey {
                        columns,
                        referenced_table,
                        referenced_columns,
                        ..
                    } => {
                        assert_eq!(columns, &vec!["user_id".to_string()]);
                        assert_eq!(referenced_table, "users");
                        assert_eq!(referenced_columns, &vec!["id".to_string()]);
                    }
                    _ => panic!("Expected ForeignKey constraint"),
                }
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_create_with_primary_key() {
        let result = parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "users");
                assert!(!c.columns[0].nullable);
                assert!(c.columns[0].primary_key);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_create_with_not_null() {
        let result = parse("CREATE TABLE users (id INTEGER NOT NULL, name TEXT)");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "users");
                assert!(!c.columns[0].nullable);
                assert!(!c.columns[0].primary_key);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }
}
