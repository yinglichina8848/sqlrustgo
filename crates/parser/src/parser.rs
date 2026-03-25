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
    SetOperation(SetOperation),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    AlterTable(AlterTableStatement),
    CreateIndex(CreateIndexStatement),
    DropIndex(DropIndexStatement),
    CreateView(CreateViewStatement),
    Analyze(AnalyzeStatement),
    Explain(ExplainStatement),
    Transaction(TransactionStatement),
    Grant(GrantStatement),
    Revoke(RevokeStatement),
}

/// CREATE INDEX statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndexStatement {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// DROP INDEX statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropIndexStatement {
    pub name: String,
}

/// ALTER TABLE operation type
#[derive(Debug, Clone, PartialEq)]
pub enum AlterOperation {
    AddColumn { name: String, data_type: String },
    DropColumn { name: String },
    ModifyColumn { name: String, data_type: String },
}

/// ALTER TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStatement {
    pub table: String,
    pub operation: AlterOperation,
}

/// Set operation type for UNION, INTERSECT, EXCEPT
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperationType {
    Union,
    UnionAll,
    Intersect,
    Except,
}

/// Set operation combining multiple SELECT statements
#[derive(Debug, Clone, PartialEq)]
pub struct SetOperation {
    pub op_type: SetOperationType,
    pub left: Box<SelectStatement>,
    pub right: Box<SelectStatement>,
}

/// ANALYZE statement for collecting statistics
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzeStatement {
    pub table_name: Option<String>,
}

/// EXPLAIN statement for showing execution plan
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainStatement {
    pub query: Box<Statement>,
    pub analyze: bool,
}

/// Transaction statement
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionStatement {
    pub command: TransactionCommand,
}

/// Transaction command types
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionCommand {
    Begin,
    Commit,
    Rollback,
    Savepoint { name: String },
    RollbackTo { name: String },
}

/// GRANT statement for permission management
#[derive(Debug, Clone, PartialEq)]
pub struct GrantStatement {
    pub privilege: Privilege,
    pub table: String,
    pub to_user: String,
}

/// REVOKE statement for permission management
#[derive(Debug, Clone, PartialEq)]
pub struct RevokeStatement {
    pub privilege: Privilege,
    pub table: String,
    pub from_user: String,
}

/// Privilege types
#[derive(Debug, Clone, PartialEq)]
pub enum Privilege {
    Read,
    Write,
    All,
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
    pub limit: Option<usize>,
    pub offset: Option<usize>,
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
    pub on_duplicate: Option<Vec<(String, Expression)>>, // ON DUPLICATE KEY UPDATE
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

/// CREATE VIEW statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewStatement {
    pub name: String,
    pub query: String,
}

/// Column definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub references: Option<ForeignKeyRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Wildcard,
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
            Some(Token::Alter) => self.parse_alter(),
            Some(Token::Index) => self.parse_create_index(),
            Some(Token::Create) => self.parse_create_or_index(),
            Some(Token::Drop) => self.parse_drop_table(),
            Some(Token::Analyze) => self.parse_analyze(),
            Some(Token::Explain) => self.parse_explain(),
            Some(Token::Begin)
            | Some(Token::Commit)
            | Some(Token::Rollback)
            | Some(Token::Savepoint) => self.parse_transaction(),
            Some(Token::Grant) => self.parse_grant(),
            Some(Token::Revoke) => self.parse_revoke(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.expect(Token::Select)?;

        let mut columns = Vec::new();
        let mut aggregates = Vec::new();
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
                Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
                | Some(Token::Max) => {
                    let func = match self.current() {
                        Some(Token::Count) => AggregateFunction::Count,
                        Some(Token::Sum) => AggregateFunction::Sum,
                        Some(Token::Avg) => AggregateFunction::Avg,
                        Some(Token::Min) => AggregateFunction::Min,
                        Some(Token::Max) => AggregateFunction::Max,
                        _ => return Err("Unknown aggregate function".to_string()),
                    };
                    self.next(); // consume function name
                    self.expect(Token::LParen)?;

                    let mut args = Vec::new();
                    match self.current() {
                        Some(Token::Star) => {
                            args.push(Expression::Wildcard);
                            self.next();
                        }
                        Some(Token::Identifier(name)) => {
                            args.push(Expression::Identifier(name.to_string()));
                            self.next();
                        }
                        _ => return Err("Expected * or column name in aggregate".to_string()),
                    }

                    self.expect(Token::RParen)?;

                    let agg = AggregateCall {
                        func,
                        args,
                        distinct: false,
                    };
                    aggregates.push(agg);
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

        let base_select = SelectStatement {
            columns,
            table,
            where_clause,
            join_clause: None,
            aggregates,
            limit: None,
            offset: None,
        };

        // Check for LIMIT and OFFSET
        let mut select = base_select.clone();
        match self.current() {
            Some(Token::Limit) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    select.limit = Some(n.parse().unwrap_or(0));
                    self.next();
                } else {
                    return Err("Expected number after LIMIT".to_string());
                }
            }
            _ => {}
        }

        match self.current() {
            Some(Token::Offset) => {
                self.next();
                if let Some(Token::NumberLiteral(n)) = self.current() {
                    select.offset = Some(n.parse().unwrap_or(0));
                    self.next();
                } else {
                    return Err("Expected number after OFFSET".to_string());
                }
            }
            _ => {}
        }

        // Check for set operations (UNION, INTERSECT, EXCEPT)
        match self.current() {
            Some(Token::Union) => {
                self.next(); // consume UNION
                let all = matches!(self.current(), Some(Token::All));
                if all {
                    self.next(); // consume ALL
                }
                // Recursively parse the next SELECT statement
                let right = match self.parse_select()? {
                    Statement::Select(s) => Box::new(s),
                    Statement::SetOperation(_) => {
                        return Err("Nested set operations not yet supported".to_string())
                    }
                    _ => return Err("Expected SELECT after UNION".to_string()),
                };
                let op_type = if all {
                    SetOperationType::UnionAll
                } else {
                    SetOperationType::Union
                };
                Ok(Statement::SetOperation(SetOperation {
                    op_type,
                    left: Box::new(base_select),
                    right,
                }))
            }
            Some(Token::Intersect) => {
                self.next(); // consume INTERSECT
                let right = match self.parse_select()? {
                    Statement::Select(s) => Box::new(s),
                    _ => return Err("Expected SELECT after INTERSECT".to_string()),
                };
                Ok(Statement::SetOperation(SetOperation {
                    op_type: SetOperationType::Intersect,
                    left: Box::new(base_select),
                    right,
                }))
            }
            Some(Token::Except) => {
                self.next(); // consume EXCEPT
                let right = match self.parse_select()? {
                    Statement::Select(s) => Box::new(s),
                    _ => return Err("Expected SELECT after EXCEPT".to_string()),
                };
                Ok(Statement::SetOperation(SetOperation {
                    op_type: SetOperationType::Except,
                    left: Box::new(base_select),
                    right,
                }))
            }
            _ => Ok(Statement::Select(select)),
        }
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.expect(Token::Insert)?;
        self.expect(Token::Into)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Check for INSERT SET syntax or column list
        let mut columns = Vec::new();
        let mut values = Vec::new();

        if matches!(self.current(), Some(Token::Set)) {
            // INSERT INTO table SET col1=val1, col2=val2
            self.next(); // consume SET

            loop {
                // Parse column name
                let col_name = match self.current() {
                    Some(Token::Identifier(name)) => name.clone(),
                    _ => break,
                };
                self.next();
                columns.push(col_name);

                // Expect =
                match self.current() {
                    Some(Token::Equal) => {}
                    _ => return Err("Expected = after column name".to_string()),
                }
                self.next(); // consume =

                // Parse value - extract first, then advance
                let value = match self.current() {
                    Some(Token::NumberLiteral(n)) => {
                        let v = Expression::Literal(n.clone());
                        self.next();
                        v
                    }
                    Some(Token::StringLiteral(s)) => {
                        let v = Expression::Literal(format!("'{}'", s));
                        self.next();
                        v
                    }
                    Some(Token::Identifier(name)) => {
                        if name.to_uppercase() == "NULL" {
                            let v = Expression::Literal("NULL".to_string());
                            self.next();
                            v
                        } else {
                            let v = Expression::Identifier(name.clone());
                            self.next();
                            v
                        }
                    }
                    Some(Token::Minus) => {
                        self.next();
                        if let Some(Token::NumberLiteral(n)) = self.current() {
                            let v = Expression::Literal(format!("-{}", n));
                            self.next();
                            v
                        } else {
                            return Err("Expected number after -".to_string());
                        }
                    }
                    _ => return Err("Expected value".to_string()),
                };
                values.push(vec![value]);

                // Check for more columns
                match self.current() {
                    Some(Token::Comma) => {
                        self.next(); // consume comma
                    }
                    _ => break,
                }
            }

            return Ok(Statement::Insert(InsertStatement {
                table,
                columns,
                values,
                on_duplicate: None,
            }));
        } else if matches!(self.current(), Some(Token::LParen)) {
            // INSERT INTO table (col1, col2) VALUES ...
            self.next(); // consume '('
            loop {
                if let Some(Token::Identifier(name)) = self.current() {
                    columns.push(name.clone());
                    self.next();
                } else if matches!(self.current(), Some(Token::RParen)) {
                    self.next();
                    break;
                } else if matches!(self.current(), Some(Token::Comma)) {
                    self.next();
                } else {
                    return Err("Expected column name".to_string());
                }
            }
        }

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
                    Some(Token::Identifier(name)) => {
                        if name.to_uppercase() == "NULL" {
                            row.push(Expression::Literal("NULL".to_string()));
                        } else {
                            row.push(Expression::Identifier(name.clone()));
                        }
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

        // Check for ON DUPLICATE KEY UPDATE
        let mut on_duplicate = None;
        if let Some(Token::On) = self.current() {
            self.next();
            if let Some(Token::Duplicate) = self.current() {
                self.next();
                if let Some(Token::Key) = self.current() {
                    self.next();
                    if let Some(Token::Update) = self.current() {
                        self.next();
                        // Parse update clauses: col1=val1, col2=val2, ...
                        let mut updates = Vec::new();
                        loop {
                            match self.current() {
                                Some(Token::Identifier(col)) => {
                                    let col_name = col.clone();
                                    self.next();
                                    self.expect(Token::Equal)?;
                                    let expr = self.parse_expression()?;
                                    updates.push((col_name, expr));
                                }
                                Some(Token::Comma) => {
                                    self.next();
                                }
                                _ => break,
                            }
                        }
                        on_duplicate = Some(updates);
                    }
                }
            }
        }

        Ok(Statement::Insert(InsertStatement {
            table,
            columns,
            values,
            on_duplicate,
        }))
    }

    fn parse_alter(&mut self) -> Result<Statement, String> {
        self.expect(Token::Alter)?;
        self.expect(Token::Table)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        let operation = match self.current() {
            Some(Token::Add) => {
                self.next();
                self.expect(Token::Column)?;
                let name = match self.next() {
                    Some(Token::Identifier(n)) => n,
                    _ => return Err("Expected column name".to_string()),
                };
                let data_type = match self.current() {
                    Some(Token::Integer) => {
                        self.next();
                        "INTEGER".to_string()
                    }
                    Some(Token::Text) => {
                        self.next();
                        "TEXT".to_string()
                    }
                    _ => return Err("Expected data type".to_string()),
                };
                AlterOperation::AddColumn { name, data_type }
            }
            Some(Token::Drop) => {
                self.next();
                self.expect(Token::Column)?;
                let name = match self.next() {
                    Some(Token::Identifier(n)) => n,
                    _ => return Err("Expected column name".to_string()),
                };
                AlterOperation::DropColumn { name }
            }
            Some(Token::Modify) => {
                self.next();
                self.expect(Token::Column)?;
                let name = match self.next() {
                    Some(Token::Identifier(n)) => n,
                    _ => return Err("Expected column name".to_string()),
                };
                let data_type = match self.current() {
                    Some(Token::Integer) => {
                        self.next();
                        "INTEGER".to_string()
                    }
                    Some(Token::Text) => {
                        self.next();
                        "TEXT".to_string()
                    }
                    _ => return Err("Expected data type".to_string()),
                };
                AlterOperation::ModifyColumn { name, data_type }
            }
            _ => return Err("Expected ADD, DROP or MODIFY".to_string()),
        };
        Ok(Statement::AlterTable(AlterTableStatement {
            table,
            operation,
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

    /// Parse comparison expression (=, !=, >, <, >=, <=)
    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_primary_expression()?;

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

        let right = self.parse_primary_expression()?;

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    /// Parse primary expression (identifier, literal, or parenthesized)
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.current() {
            Some(Token::Identifier(name)) => {
                if name.to_uppercase() == "NULL" {
                    return Ok(Expression::Literal("NULL".to_string()));
                }
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
                // Parenthesized expression
                self.next(); // consume '('
                let expr = self.parse_or_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
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

    fn parse_create_index(&mut self) -> Result<Statement, String> {
        // CREATE [UNIQUE] INDEX index_name ON table (col1, col2, ...)
        let mut unique = false;
        if matches!(self.current(), Some(Token::Unique)) {
            unique = true;
            self.next();
        }

        if !matches!(self.current(), Some(Token::Index)) {
            return Err("Expected INDEX after CREATE or UNIQUE".to_string());
        }
        self.next();
        let name = match self.next() {
            Some(Token::Identifier(n)) => n,
            _ => return Err("Expected index name".to_string()),
        };

        self.expect(Token::On)?;
        let table = match self.next() {
            Some(Token::Identifier(t)) => t,
            _ => return Err("Expected table name".to_string()),
        };

        self.expect(Token::LParen)?;
        let mut columns = Vec::new();
        loop {
            match self.current() {
                Some(Token::Identifier(col)) => {
                    columns.push(col.clone());
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

        Ok(Statement::CreateIndex(CreateIndexStatement {
            name,
            table,
            columns,
            unique,
        }))
    }

    fn parse_drop_index(&mut self) -> Result<Statement, String> {
        // DROP INDEX index_name
        self.expect(Token::Index)?;
        let name = match self.next() {
            Some(Token::Identifier(n)) => n,
            _ => return Err("Expected index name".to_string()),
        };

        Ok(Statement::DropIndex(DropIndexStatement { name }))
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    fn parse_create_or_index(&mut self) -> Result<Statement, String> {
        let mut pos = self.position;
        let mut is_index = false;
        if pos < self.tokens.len() {
            pos += 1;
        }
        if pos < self.tokens.len() {
            if let Some(Token::Unique) = &self.tokens.get(pos) {
                is_index = true;
                pos += 1;
            }
        }
        if pos < self.tokens.len() {
            if let Some(Token::Index) = &self.tokens.get(pos) {
                is_index = true;
            }
        }
        if is_index {
            self.next();
            self.parse_create_index()
        } else {
            self.parse_create()
        }
    }

    fn parse_create(&mut self) -> Result<Statement, String> {
        self.expect(Token::Create)?;

        match self.current() {
            Some(Token::Table) => {
                self.next();
                self.parse_create_table()
            }
            Some(Token::View) => {
                self.next();
                self.parse_create_view()
            }
            _ => Err("Expected TABLE or VIEW after CREATE".to_string()),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
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

                        // Parse column constraints
                        let mut nullable = true;
                        let mut auto_increment = false;
                        let mut primary_key = false;
                        let mut references = None;

                        loop {
                            match self.current() {
                                Some(Token::Not) => {
                                    self.next();
                                    if let Some(Token::Identifier(s)) = self.current() {
                                        if s.to_uppercase() == "NULL" {
                                            nullable = false;
                                            self.next();
                                        }
                                    }
                                }
                                Some(Token::AutoIncrement) => {
                                    auto_increment = true;
                                    self.next();
                                }
                                Some(Token::Primary) => {
                                    self.next();
                                    if let Some(Token::Key) = self.current() {
                                        primary_key = true;
                                        self.next();
                                    }
                                }
                                Some(Token::References) => {
                                    self.next();
                                    let ref_table = match self.current() {
                                        Some(Token::Identifier(s)) => {
                                            let t = s.clone();
                                            self.next();
                                            t
                                        }
                                        _ => "".to_string(),
                                    };
                                    let ref_column = match self.current() {
                                        Some(Token::Identifier(s)) => {
                                            let c = s.clone();
                                            self.next();
                                            c
                                        }
                                        _ => "id".to_string(),
                                    };
                                    references = Some(ForeignKeyRef {
                                        table: ref_table,
                                        column: ref_column,
                                        on_delete: None,
                                        on_update: None,
                                    });
                                }
                                _ => break,
                            }
                        }

                        columns.push(ColumnDefinition {
                            name: col_name,
                            data_type,
                            nullable,
                            auto_increment,
                            primary_key,
                            references,
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

    fn parse_create_view(&mut self) -> Result<Statement, String> {
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected view name".to_string()),
        };

        self.expect(Token::As)?;

        let mut query_parts = Vec::new();
        while let Some(token) = self.current() {
            match token {
                Token::Semicolon => {
                    break;
                }
                _ => {
                    query_parts.push(token.to_string());
                    self.next();
                }
            }
        }

        let query = query_parts.join(" ");
        Ok(Statement::CreateView(CreateViewStatement { name, query }))
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

    fn parse_explain(&mut self) -> Result<Statement, String> {
        self.expect(Token::Explain)?;

        let analyze = match self.current() {
            Some(Token::Analyze) => {
                self.next();
                true
            }
            _ => false,
        };

        let query = match self.current() {
            Some(Token::Select) => Box::new(self.parse_select()?),
            _ => return Err("EXPLAIN must be followed by a SELECT statement".to_string()),
        };

        Ok(Statement::Explain(ExplainStatement { query, analyze }))
    }

    fn parse_grant(&mut self) -> Result<Statement, String> {
        self.expect(Token::Grant)?;

        let privilege = match self.current() {
            Some(Token::Select) => {
                self.next();
                Privilege::Read
            }
            Some(Token::Insert) | Some(Token::Update) | Some(Token::Delete) => {
                self.next();
                Privilege::Write
            }
            Some(Token::All) => {
                self.next();
                Privilege::All
            }
            Some(Token::Identifier(s)) => {
                let upper = s.to_uppercase();
                self.next();
                match upper.as_str() {
                    "READ" => Privilege::Read,
                    "WRITE" => Privilege::Write,
                    "ALL" => Privilege::All,
                    _ => return Err("Expected privilege (READ, WRITE, or ALL)".to_string()),
                }
            }
            _ => return Err("Expected privilege (READ, WRITE, or ALL)".to_string()),
        };

        self.expect(Token::On)?;

        let table = match self.current() {
            Some(Token::Identifier(name)) => {
                let t = name.clone();
                self.next();
                t
            }
            _ => return Err("Expected table name".to_string()),
        };

        self.expect(Token::To)?;

        let to_user = match self.current() {
            Some(Token::Identifier(name)) => {
                let u = name.clone();
                self.next();
                u
            }
            _ => return Err("Expected user name".to_string()),
        };

        Ok(Statement::Grant(GrantStatement {
            privilege,
            table,
            to_user,
        }))
    }

    fn parse_revoke(&mut self) -> Result<Statement, String> {
        self.expect(Token::Revoke)?;

        let privilege = match self.current() {
            Some(Token::Select) => {
                self.next();
                Privilege::Read
            }
            Some(Token::Insert) | Some(Token::Update) | Some(Token::Delete) => {
                self.next();
                Privilege::Write
            }
            Some(Token::All) => {
                self.next();
                Privilege::All
            }
            Some(Token::Identifier(s)) => {
                let upper = s.to_uppercase();
                self.next();
                match upper.as_str() {
                    "READ" => Privilege::Read,
                    "WRITE" => Privilege::Write,
                    "ALL" => Privilege::All,
                    _ => return Err("Expected privilege (READ, WRITE, or ALL)".to_string()),
                }
            }
            _ => return Err("Expected privilege (READ, WRITE, or ALL)".to_string()),
        };

        self.expect(Token::On)?;

        let table = match self.current() {
            Some(Token::Identifier(name)) => {
                let t = name.clone();
                self.next();
                t
            }
            _ => return Err("Expected table name".to_string()),
        };

        self.expect(Token::From)?;

        let from_user = match self.current() {
            Some(Token::Identifier(name)) => {
                let u = name.clone();
                self.next();
                u
            }
            _ => return Err("Expected user name".to_string()),
        };

        Ok(Statement::Revoke(RevokeStatement {
            privilege,
            table,
            from_user,
        }))
    }

    fn parse_transaction(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Begin) => {
                self.next();
                Ok(Statement::Transaction(TransactionStatement {
                    command: TransactionCommand::Begin,
                }))
            }
            Some(Token::Commit) => {
                self.next();
                Ok(Statement::Transaction(TransactionStatement {
                    command: TransactionCommand::Commit,
                }))
            }
            Some(Token::Rollback) => {
                self.next();
                match self.current() {
                    Some(Token::Savepoint) => {
                        self.next();
                        let name = match self.current() {
                            Some(Token::Identifier(n)) => {
                                let name = n.clone();
                                self.next();
                                name
                            }
                            _ => return Err("Expected savepoint name".to_string()),
                        };
                        Ok(Statement::Transaction(TransactionStatement {
                            command: TransactionCommand::RollbackTo { name },
                        }))
                    }
                    _ => Ok(Statement::Transaction(TransactionStatement {
                        command: TransactionCommand::Rollback,
                    })),
                }
            }
            Some(Token::Savepoint) => {
                self.next();
                let name = match self.current() {
                    Some(Token::Identifier(n)) => {
                        let name = n.clone();
                        self.next();
                        name
                    }
                    _ => return Err("Expected savepoint name".to_string()),
                };
                Ok(Statement::Transaction(TransactionStatement {
                    command: TransactionCommand::Savepoint { name },
                }))
            }
            _ => Err("Invalid transaction statement".to_string()),
        }
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

    #[test]
    fn test_parse_select_star() {
        let result = parse("SELECT * FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.columns.len(), 1);
                assert_eq!(s.columns[0].name, "*");
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_select_multiple_columns() {
        let result = parse("SELECT id, name, age FROM users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 3);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_select_with_where() {
        let result = parse("SELECT id FROM users WHERE name = 'Alice'");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Select(s) => {
                assert!(s.where_clause.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_analyze_with_table() {
        let result = parse("ANALYZE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Analyze(a) => {
                assert_eq!(a.table_name, Some("users".to_string()));
            }
            _ => panic!("Expected ANALYZE statement"),
        }
    }

    #[test]
    fn test_parse_error_empty() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = parse("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_select_missing_from() {
        let result = parse("SELECT id");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_insert_missing_table() {
        let result = parse("INSERT VALUES (1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_is_eof() {
        let tokens = vec![Token::Eof];
        let parser = Parser::new(tokens);
        assert!(parser.is_eof());
    }

    #[test]
    fn test_parser_expect() {
        let tokens = vec![Token::Select, Token::From];
        let mut parser = Parser::new(tokens);
        let result = parser.expect(Token::Select);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_expect_error() {
        let tokens = vec![Token::From];
        let mut parser = Parser::new(tokens);
        let result = parser.expect(Token::Select);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_table_with_not_null() {
        let result = parse("CREATE TABLE users (id INTEGER NOT NULL, name TEXT)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_with_multiple_columns() {
        let result = parse("CREATE TABLE test (a INTEGER, b TEXT, c FLOAT, d BOOLEAN)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_multi_set() {
        let result = parse("UPDATE users SET name = 'A', age = 30 WHERE id = 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_insert_error_no_table() {
        let result = parse("INSERT VALUES (1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_insert_error_no_values() {
        let result = parse("INSERT INTO users ()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_select_all_columns() {
        let result = parse("SELECT * FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_delete_error_no_table() {
        let result = parse("DELETE WHERE id = 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_update_error_no_set() {
        let result = parse("UPDATE users WHERE id = 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_create_without_columns() {
        let result = parse("CREATE TABLE users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_drop_error_no_name() {
        let result = parse("DROP TABLE");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_insert_single_value() {
        let result = parse("INSERT INTO users VALUES (1)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_insert_with_column_list() {
        let result = parse("INSERT INTO users (id, name) VALUES (1, 'test')");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.columns, vec!["id", "name"]);
            }
            _ => panic!("Expected INSERT"),
        }
    }

    #[test]
    fn test_parse_update_single_set() {
        let result = parse("UPDATE users SET name = 'Bob'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_explain() {
        let result = parse("EXPLAIN SELECT id FROM users");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Explain(e) => {
                assert!(!e.analyze);
                match *e.query {
                    Statement::Select(s) => {
                        assert_eq!(s.table, "users");
                    }
                    _ => panic!("Expected SELECT in EXPLAIN"),
                }
            }
            _ => panic!("Expected EXPLAIN statement"),
        }
    }

    #[test]
    fn test_parse_explain_analyze() {
        let result = parse("EXPLAIN ANALYZE SELECT id FROM users");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Explain(e) => {
                assert!(e.analyze);
            }
            _ => panic!("Expected EXPLAIN statement"),
        }
    }

    #[test]
    fn test_parse_grant_read() {
        let result = parse("GRANT READ ON users TO alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Read);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "alice");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_write() {
        let result = parse("GRANT WRITE ON orders TO bob");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Write);
                assert_eq!(g.table, "orders");
                assert_eq!(g.to_user, "bob");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_all() {
        let result = parse("GRANT ALL ON products TO admin");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::All);
                assert_eq!(g.table, "products");
                assert_eq!(g.to_user, "admin");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_select() {
        let result = parse("GRANT SELECT ON users TO guest");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Read);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "guest");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_insert() {
        let result = parse("GRANT INSERT ON users TO writer");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Write);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "writer");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_update() {
        let result = parse("GRANT UPDATE ON users TO updater");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Write);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "updater");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_delete() {
        let result = parse("GRANT DELETE ON users TO deleter");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Write);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "deleter");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_lowercase() {
        let result = parse("grant read on users to alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Read);
                assert_eq!(g.table, "users");
                assert_eq!(g.to_user, "alice");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_mixed_case() {
        let result = parse("Grant Read On Users To Alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Grant(g) => {
                assert_eq!(g.privilege, Privilege::Read);
                assert_eq!(g.table, "Users");
                assert_eq!(g.to_user, "Alice");
            }
            _ => panic!("Expected GRANT statement"),
        }
    }

    #[test]
    fn test_parse_grant_invalid_privilege() {
        let result = parse("GRANT INVALID ON users TO alice");
        assert!(result.is_err(), "Expected error for invalid privilege");
    }

    #[test]
    fn test_parse_grant_missing_on() {
        let result = parse("GRANT READ users TO alice");
        assert!(result.is_err(), "Expected error for missing ON");
    }

    #[test]
    fn test_parse_grant_missing_to() {
        let result = parse("GRANT READ ON users alice");
        assert!(result.is_err(), "Expected error for missing TO");
    }

    #[test]
    fn test_parse_revoke_read() {
        let result = parse("REVOKE READ ON users FROM alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Read);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "alice");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_write() {
        let result = parse("REVOKE WRITE ON orders FROM bob");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Write);
                assert_eq!(r.table, "orders");
                assert_eq!(r.from_user, "bob");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_all() {
        let result = parse("REVOKE ALL ON products FROM admin");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::All);
                assert_eq!(r.table, "products");
                assert_eq!(r.from_user, "admin");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_select() {
        let result = parse("REVOKE SELECT ON users FROM guest");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Read);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "guest");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_insert() {
        let result = parse("REVOKE INSERT ON users FROM writer");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Write);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "writer");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_update() {
        let result = parse("REVOKE UPDATE ON users FROM updater");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Write);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "updater");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_delete() {
        let result = parse("REVOKE DELETE ON users FROM deleter");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Write);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "deleter");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_lowercase() {
        let result = parse("revoke read on users from alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Read);
                assert_eq!(r.table, "users");
                assert_eq!(r.from_user, "alice");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_mixed_case() {
        let result = parse("Revoke Read On Users From Alice");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Revoke(r) => {
                assert_eq!(r.privilege, Privilege::Read);
                assert_eq!(r.table, "Users");
                assert_eq!(r.from_user, "Alice");
            }
            _ => panic!("Expected REVOKE statement"),
        }
    }

    #[test]
    fn test_parse_revoke_invalid_privilege() {
        let result = parse("REVOKE INVALID ON users FROM alice");
        assert!(result.is_err(), "Expected error for invalid privilege");
    }

    #[test]
    fn test_parse_revoke_missing_on() {
        let result = parse("REVOKE READ users FROM alice");
        assert!(result.is_err(), "Expected error for missing ON");
    }

    #[test]
    fn test_parse_revoke_missing_from() {
        let result = parse("REVOKE READ ON users alice");
        assert!(result.is_err(), "Expected error for missing FROM");
    }

    #[test]
    fn test_privilege_enum() {
        assert_eq!(format!("{:?}", Privilege::Read), "Read");
        assert_eq!(format!("{:?}", Privilege::Write), "Write");
        assert_eq!(format!("{:?}", Privilege::All), "All");
    }

    #[test]
    fn test_grant_statement_clone() {
        let grant = GrantStatement {
            privilege: Privilege::Read,
            table: "users".to_string(),
            to_user: "alice".to_string(),
        };
        let cloned = grant.clone();
        assert_eq!(grant.privilege, cloned.privilege);
        assert_eq!(grant.table, cloned.table);
        assert_eq!(grant.to_user, cloned.to_user);
    }

    #[test]
    fn test_revoke_statement_clone() {
        let revoke = RevokeStatement {
            privilege: Privilege::Write,
            table: "orders".to_string(),
            from_user: "bob".to_string(),
        };
        let cloned = revoke.clone();
        assert_eq!(revoke.privilege, cloned.privilege);
        assert_eq!(revoke.table, cloned.table);
        assert_eq!(revoke.from_user, cloned.from_user);
    }

    #[test]
    fn test_grant_statement_debug() {
        let grant = GrantStatement {
            privilege: Privilege::Read,
            table: "users".to_string(),
            to_user: "alice".to_string(),
        };
        let debug = format!("{:?}", grant);
        assert!(debug.contains("GrantStatement"));
        assert!(debug.contains("Read"));
        assert!(debug.contains("users"));
        assert!(debug.contains("alice"));
    }

    #[test]
    fn test_revoke_statement_debug() {
        let revoke = RevokeStatement {
            privilege: Privilege::Write,
            table: "orders".to_string(),
            from_user: "bob".to_string(),
        };
        let debug = format!("{:?}", revoke);
        assert!(debug.contains("RevokeStatement"));
        assert!(debug.contains("Write"));
        assert!(debug.contains("orders"));
        assert!(debug.contains("bob"));
    }

    #[test]
    fn test_multiple_grant_statements() {
        let grants = vec![
            "GRANT READ ON users TO alice",
            "GRANT WRITE ON orders TO bob",
            "GRANT ALL ON admin TO root",
        ];
        for sql in grants {
            let result = parse(sql);
            assert!(result.is_ok(), "Failed to parse: {}", sql);
        }
    }

    #[test]
    fn test_multiple_revoke_statements() {
        let revokes = vec![
            "REVOKE READ ON users FROM alice",
            "REVOKE WRITE ON orders FROM bob",
            "REVOKE ALL ON admin FROM root",
        ];
        for sql in revokes {
            let result = parse(sql);
            assert!(result.is_ok(), "Failed to parse: {}", sql);
        }
    }

    #[test]
    fn test_parse_upsert_single() {
        let result = parse(
            "INSERT INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name='Alice'",
        );
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert!(i.on_duplicate.is_some());
                let updates = i.on_duplicate.unwrap();
                assert_eq!(updates.len(), 1);
                assert_eq!(updates[0].0, "name");
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_upsert_multiple() {
        let result = parse("INSERT INTO users (id, name) VALUES (1, 'A') ON DUPLICATE KEY UPDATE name='A', id=id+1");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert!(i.on_duplicate.is_some());
                let updates = i.on_duplicate.unwrap();
                assert_eq!(updates.len(), 2);
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_upsert_without_on_duplicate() {
        let result = parse("INSERT INTO users (id, name) VALUES (1, 'Alice')");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert!(i.on_duplicate.is_none());
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_batch_insert() {
        let result = parse("INSERT INTO users VALUES (1, 'A'), (2, 'B'), (3, 'C')");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.values.len(), 3);
                assert_eq!(i.table, "users");
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_column_auto_increment() {
        let result = parse("CREATE TABLE orders (id INTEGER AUTO_INCREMENT, name TEXT)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 2);
                assert!(ct.columns[0].auto_increment);
                assert!(!ct.columns[1].auto_increment);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_primary_key() {
        let result = parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 2);
                assert!(ct.columns[0].primary_key);
                assert!(!ct.columns[1].primary_key);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_foreign_key() {
        let result =
            parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 2);
                assert!(ct.columns[1].references.is_some());
                let fk = ct.columns[1].references.as_ref().unwrap();
                assert_eq!(fk.table, "users");
                assert_eq!(fk.column, "id");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_not_null() {
        let result = parse("CREATE TABLE users (id INTEGER NOT NULL, name TEXT)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 2);
                assert!(!ct.columns[0].nullable);
                assert!(ct.columns[1].nullable);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_all_constraints() {
        let result = parse("CREATE TABLE orders (id INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL, user_id INTEGER REFERENCES users(id))");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 2);
                let col = &ct.columns[0];
                assert!(col.primary_key);
                assert!(col.auto_increment);
                assert!(!col.nullable);
                assert!(ct.columns[1].references.is_some());
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_create_view() {
        let result = parse("CREATE VIEW user_view AS SELECT * FROM users");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateView(cv) => {
                assert_eq!(cv.name, "user_view");
                assert!(cv.query.contains("SELECT"));
            }
            _ => panic!("Expected CREATE VIEW statement"),
        }
    }

    #[test]
    fn test_parse_create_view_complex() {
        let result =
            parse("CREATE VIEW active_users AS SELECT id, name FROM users WHERE active = true");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateView(cv) => {
                assert_eq!(cv.name, "active_users");
                assert!(cv.query.contains("WHERE"));
            }
            _ => panic!("Expected CREATE VIEW statement"),
        }
    }

    #[test]
    fn test_column_definition_clone() {
        let col = ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            auto_increment: true,
            primary_key: true,
            references: Some(ForeignKeyRef {
                table: "users".to_string(),
                column: "id".to_string(),
                on_delete: None,
                on_update: None,
            }),
        };
        let cloned = col.clone();
        assert_eq!(col.name, cloned.name);
        assert_eq!(col.auto_increment, cloned.auto_increment);
        assert_eq!(col.primary_key, cloned.primary_key);
    }

    #[test]
    fn test_column_definition_debug() {
        let col = ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            auto_increment: true,
            primary_key: false,
            references: None,
        };
        let debug = format!("{:?}", col);
        assert!(debug.contains("id"));
        assert!(debug.contains("INTEGER"));
        assert!(debug.contains("auto_increment"));
    }

    #[test]
    fn test_insert_statement_with_on_duplicate_clone() {
        let insert = InsertStatement {
            table: "users".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![vec![Expression::Literal("1".to_string())]],
            on_duplicate: Some(vec![(
                "name".to_string(),
                Expression::Literal("new".to_string()),
            )]),
        };
        let cloned = insert.clone();
        assert_eq!(insert.table, cloned.table);
        assert!(cloned.on_duplicate.is_some());
    }

    #[test]
    fn test_foreign_key_ref_clone() {
        let fk = ForeignKeyRef {
            table: "orders".to_string(),
            column: "user_id".to_string(),
            on_delete: Some(ForeignKeyAction::Cascade),
            on_update: Some(ForeignKeyAction::Restrict),
        };
        let cloned = fk.clone();
        assert_eq!(fk.table, cloned.table);
        assert_eq!(fk.on_delete, cloned.on_delete);
    }

    #[test]
    fn test_foreign_key_action_enum() {
        assert_eq!(format!("{:?}", ForeignKeyAction::Cascade), "Cascade");
        assert_eq!(format!("{:?}", ForeignKeyAction::SetNull), "SetNull");
        assert_eq!(format!("{:?}", ForeignKeyAction::Restrict), "Restrict");
    }

    #[test]
    fn test_create_view_statement_clone() {
        let cv = CreateViewStatement {
            name: "my_view".to_string(),
            query: "SELECT * FROM users".to_string(),
        };
        let cloned = cv.clone();
        assert_eq!(cv.name, cloned.name);
        assert_eq!(cv.query, cloned.query);
    }

    #[test]
    fn test_create_view_statement_debug() {
        let cv = CreateViewStatement {
            name: "my_view".to_string(),
            query: "SELECT * FROM users".to_string(),
        };
        let debug = format!("{:?}", cv);
        assert!(debug.contains("my_view"));
        assert!(debug.contains("SELECT"));
    }

    #[test]
    fn test_parse_view_lowercase() {
        let result = parse("create view my_view as select * from users");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_view_missing_as() {
        let result = parse("CREATE VIEW my_view SELECT * FROM users");
        assert!(result.is_err(), "Expected error for missing AS");
    }

    #[test]
    fn test_parse_upsert_empty_update() {
        let result = parse("INSERT INTO users VALUES (1) ON DUPLICATE KEY UPDATE ");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_batch_insert_single_row() {
        let result = parse("INSERT INTO users VALUES (1, 'Alice')");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_batch_insert_empty_values() {
        let result = parse("INSERT INTO users VALUES ()");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_column_foreign_key_no_column() {
        let result = parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                let fk = ct.columns[1].references.as_ref().unwrap();
                assert_eq!(fk.table, "users");
                assert_eq!(fk.column, "id");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_auto_increment_synonyms() {
        let result = parse("CREATE TABLE t1 (id INTEGER AUTOINCREMENT)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }
}
