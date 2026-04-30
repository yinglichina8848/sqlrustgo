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
use crate::transaction::{IsolationLevel, TransactionStatement};
use serde::{Deserialize, Serialize};

/// SQL Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    CreateIndex(CreateIndexStatement),
    DropTable(DropTableStatement),
    Truncate(TruncateStatement),
    Analyze(AnalyzeStatement),
    WithSelect(WithSelect),
    AlterTable(AlterTableStatement),
    Call(CallStatement),
    CreateProcedure(CreateProcedureStatement),
    Union(UnionStatement),
    CreateTrigger(CreateTriggerStatement),
    Transaction(TransactionStatement),
    Grant(GrantStatement),
    Revoke(RevokeStatement),
    Show(ShowStatement),
    Describe(DescribeStatement),
}

/// UNION statement
#[derive(Debug, Clone, PartialEq)]
pub struct UnionStatement {
    pub left: Box<Statement>,
    pub right: Box<Statement>,
    pub union_all: bool,
}

/// CREATE INDEX statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndexStatement {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// ALTER TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableStatement {
    pub table_name: String,
    pub operation: AlterTableOperation,
}

/// ALTER TABLE operation types
#[derive(Debug, Clone, PartialEq)]
pub enum AlterTableOperation {
    AddColumn {
        name: String,
        data_type: String,
        nullable: bool,
        default_value: Option<String>,
    },
    RenameTo {
        new_name: String,
    },
}

/// CALL statement for invoking stored procedures
#[derive(Debug, Clone, PartialEq)]
pub struct CallStatement {
    pub procedure_name: String,
    pub args: Vec<String>,
}

/// CREATE PROCEDURE statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateProcedureStatement {
    pub name: String,
    pub params: Vec<StoredProcParam>,
    pub body: Vec<StoredProcStatement>,
}

/// Stored procedure parameter
#[derive(Debug, Clone, PartialEq)]
pub struct StoredProcParam {
    pub name: String,
    pub mode: StoredProcParamMode,
    pub data_type: String,
}

/// Parameter mode for stored procedure
#[derive(Debug, Clone, PartialEq)]
pub enum StoredProcParamMode {
    In,
    Out,
    InOut,
}

/// Stored procedure statement types
#[derive(Debug, Clone, PartialEq)]
pub enum StoredProcStatement {
    RawSql(String),
}

/// CREATE TRIGGER statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTriggerStatement {
    pub name: String,
    pub table: String,
    pub timing: String,
    pub events: Vec<String>,
    pub body: String,
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

#[derive(Debug, Clone, PartialEq)]
pub struct GrantStatement {
    pub privileges: Vec<Privilege>,
    pub columns: Vec<String>,
    pub object_type: ObjectType,
    pub object_name: String,
    pub recipients: Vec<String>,
    pub with_grant_option: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RevokeStatement {
    pub privileges: Vec<Privilege>,
    pub columns: Vec<String>,
    pub object_type: ObjectType,
    pub object_name: String,
    pub from_users: Vec<String>,
    pub grant_option_for: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Privilege {
    Select,
    Insert,
    Update,
    Delete,
    Read,
    Write,
    Execute,
    Usage,
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Table,
    Database,
    Procedure,
    Function,
    Column,
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
    pub on_clause: Expression,
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
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderByExpression>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct: bool,
}

/// ORDER BY expression
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByExpression {
    pub expression: Expression,
    pub ascending: bool,
}

/// Column in SELECT
#[derive(Debug, Clone, PartialEq)]
pub struct SelectColumn {
    pub name: String,
    pub alias: Option<String>,
    pub expression: Option<Expression>,
}

/// INSERT statement
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,         // For INSERT VALUES
    pub select: Option<Box<SelectStatement>>, // For INSERT SELECT
    pub is_replace: bool,                     // For REPLACE INTO (MySQL compatibility)
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

/// TRUNCATE TABLE statement
#[derive(Debug, Clone, PartialEq)]
pub struct TruncateStatement {
    pub name: String,
}

/// SHOW statement variants
#[derive(Debug, Clone, PartialEq)]
pub enum ShowStatement {
    Tables,
    Columns {
        table: String,
        pattern: Option<String>,
    },
    Index {
        table: String,
    },
    Grants {
        user: Option<String>,
    },
}

/// DESCRIBE statement (aliased as DESC)
#[derive(Debug, Clone, PartialEq)]
pub struct DescribeStatement {
    pub table: String,
}

/// Column definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
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
    InList(Box<Expression>, Vec<Expression>),  // MySQL: col IN (1, 2, 3)
    NotInList(Box<Expression>, Vec<Expression>),  // MySQL: col NOT IN (1, 2, 3)
    Exists(Box<SelectStatement>),
    NotExists(Box<SelectStatement>),
    QuantifiedOp(Box<Expression>, String, Box<SelectStatement>),
    Aggregate(AggregateCall), // For HAVING clause - supports aggregate functions in expressions
    IsNull(Box<Expression>),  // IS NULL check
    IsNotNull(Box<Expression>), // IS NOT NULL check
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
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
            Some(Token::Insert) | Some(Token::Replace) => self.parse_insert(),
            Some(Token::Update) => self.parse_update(),
            Some(Token::Delete) => self.parse_delete(),
            Some(Token::Create) => self.parse_create(),
            Some(Token::Drop) => self.parse_drop_table(),
            Some(Token::Truncate) => self.parse_truncate(),
            Some(Token::Analyze) => self.parse_analyze(),
            Some(Token::With) => self.parse_with_select(),
            Some(Token::Alter) => self.parse_alter_table(),
            Some(Token::Call) => self.parse_call(),
            Some(Token::Begin)
            | Some(Token::Commit)
            | Some(Token::Rollback)
            | Some(Token::Set)
            | Some(Token::Start) => self.parse_transaction(),
            Some(Token::Grant) => self.parse_grant(),
            Some(Token::Revoke) => self.parse_revoke(),
            Some(Token::Show) => self.parse_show(),
            Some(Token::Describe) => self.parse_describe(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_transaction(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Begin) => self.parse_begin(),
            Some(Token::Commit) => self.parse_commit(),
            Some(Token::Rollback) => self.parse_rollback(),
            Some(Token::Set) => self.parse_set_transaction(),
            Some(Token::Start) => self.parse_start_transaction(),
            Some(t) => Err(format!("Unexpected transaction token: {:?}", t)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_begin(&mut self) -> Result<Statement, String> {
        self.expect(Token::Begin)?;
        let work = if self.current() == Some(&Token::Work) {
            self.next();
            true
        } else {
            false
        };
        let isolation_level = if self.current() == Some(&Token::Isolation) {
            self.next();
            self.expect(Token::Level)?;
            Some(self.parse_isolation_level_value()?)
        } else if self.current() == Some(&Token::Serializable) {
            self.next();
            Some(IsolationLevel::Serializable)
        } else if self.current() == Some(&Token::Repeatable) {
            self.next();
            Some(IsolationLevel::SnapshotIsolation)
        } else if self.current() == Some(&Token::Read) {
            self.next();
            match self.current() {
                Some(Token::Committed) => {
                    self.next();
                    Some(IsolationLevel::ReadCommitted)
                }
                Some(Token::Uncommitted) => {
                    self.next();
                    Some(IsolationLevel::ReadUncommitted)
                }
                Some(t) => {
                    return Err(format!(
                        "Expected COMMITTED or UNCOMMITTED after READ, got {:?}",
                        t
                    ))
                }
                None => return Err("Unexpected end of input after READ".to_string()),
            }
        } else {
            None
        };
        Ok(Statement::Transaction(TransactionStatement::Begin {
            work,
            isolation_level,
        }))
    }

    fn parse_commit(&mut self) -> Result<Statement, String> {
        self.expect(Token::Commit)?;
        let work = if self.current() == Some(&Token::Work) {
            self.next();
            true
        } else {
            false
        };
        Ok(Statement::Transaction(TransactionStatement::Commit {
            work,
        }))
    }

    fn parse_rollback(&mut self) -> Result<Statement, String> {
        self.expect(Token::Rollback)?;
        let work = if self.current() == Some(&Token::Work) {
            self.next();
            true
        } else {
            false
        };
        Ok(Statement::Transaction(TransactionStatement::Rollback {
            work,
        }))
    }

    fn parse_start_transaction(&mut self) -> Result<Statement, String> {
        self.expect(Token::Start)?;
        self.expect(Token::Transaction)?;
        let isolation_level = if self.current() == Some(&Token::Isolation) {
            self.next();
            self.expect(Token::Level)?;
            Some(self.parse_isolation_level_value()?)
        } else {
            None
        };
        Ok(Statement::Transaction(
            TransactionStatement::StartTransaction { isolation_level },
        ))
    }

    fn parse_set_transaction(&mut self) -> Result<Statement, String> {
        self.expect(Token::Set)?;
        self.expect(Token::Transaction)?;
        self.expect(Token::Isolation)?;
        self.expect(Token::Level)?;
        let isolation_level = self.parse_isolation_level_value()?;
        Ok(Statement::Transaction(
            TransactionStatement::SetTransaction { isolation_level },
        ))
    }

    fn parse_isolation_level_value(&mut self) -> Result<IsolationLevel, String> {
        match self.current() {
            Some(Token::Serializable) => {
                self.next();
                Ok(IsolationLevel::Serializable)
            }
            Some(Token::Repeatable) => {
                self.next();
                Ok(IsolationLevel::SnapshotIsolation)
            }
            Some(Token::Read) => {
                self.next();
                match self.current() {
                    Some(Token::Committed) => {
                        self.next();
                        Ok(IsolationLevel::ReadCommitted)
                    }
                    Some(Token::Uncommitted) => {
                        self.next();
                        Ok(IsolationLevel::ReadUncommitted)
                    }
                    Some(t) => Err(format!(
                        "Expected COMMITTED or UNCOMMITTED after READ, got {:?}",
                        t
                    )),
                    None => Err("Unexpected end of input after READ".to_string()),
                }
            }
            Some(t) => Err(format!("Unexpected isolation level keyword: {:?}", t)),
            None => Err("Unexpected end of input after READ".to_string()),
        }
    }

    fn parse_create(&mut self) -> Result<Statement, String> {
        self.expect(Token::Create)?;
        match self.current() {
            Some(Token::Table) => self.parse_create_table(),
            Some(Token::Index) | Some(Token::Unique) => self.parse_create_index(),
            Some(Token::Procedure) => self.parse_create_procedure(),
            Some(Token::Trigger) => self.parse_create_trigger(),
            Some(t) => Err(format!(
                "Expected TABLE, INDEX, PROCEDURE, or TRIGGER after CREATE, got {:?}",
                t
            )),
            None => Err("Expected TABLE, INDEX, PROCEDURE, or TRIGGER after CREATE".to_string()),
        }
    }

    fn parse_create_index(&mut self) -> Result<Statement, String> {
        self.expect(Token::Index)?;
        let index_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected index name, got {:?}", t)),
            None => return Err("Expected index name".to_string()),
        };
        self.expect(Token::On)?;
        let table_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected table name, got {:?}", t)),
            None => return Err("Expected table name".to_string()),
        };
        self.expect(Token::LParen)?;
        let columns = self.parse_column_list()?;
        Ok(Statement::CreateIndex(CreateIndexStatement {
            name: index_name,
            table: table_name,
            columns,
            unique: false,
        }))
    }

    fn parse_create_procedure(&mut self) -> Result<Statement, String> {
        self.expect(Token::Procedure)?;

        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected procedure name, got {:?}", t)),
            None => return Err("Expected procedure name".to_string()),
        };

        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.current(), Some(Token::RParen) | None) {
            let mode = match self.current() {
                Some(Token::In) => {
                    self.next();
                    StoredProcParamMode::In
                }
                Some(Token::Identifier(mode_str))
                    if ["OUT", "INOUT"].contains(&mode_str.to_uppercase().as_str()) =>
                {
                    let mode = match mode_str.to_uppercase().as_str() {
                        "OUT" => StoredProcParamMode::Out,
                        "INOUT" => StoredProcParamMode::InOut,
                        _ => StoredProcParamMode::In,
                    };
                    self.next();
                    mode
                }
                _ => StoredProcParamMode::In,
            };

            let param_name = match self.next() {
                Some(Token::Identifier(name)) => name,
                Some(t) => return Err(format!("Expected parameter name, got {:?}", t)),
                None => return Err("Expected parameter name".to_string()),
            };

            let data_type = match self.next() {
                Some(Token::Identifier(typename)) => typename,
                Some(Token::Integer) => "INTEGER".to_string(),
                Some(Token::Text) => "TEXT".to_string(),
                Some(Token::Float) => "FLOAT".to_string(),
                Some(Token::Boolean) => "BOOLEAN".to_string(),
                Some(t) => return Err(format!("Expected data type, got {:?}", t)),
                None => return Err("Expected data type".to_string()),
            };

            params.push(StoredProcParam {
                name: param_name,
                mode,
                data_type,
            });

            if matches!(self.current(), Some(Token::Comma)) {
                self.next();
            }
        }
        self.expect(Token::RParen)?;

        self.expect(Token::Begin)?;
        let mut body = Vec::new();
        let mut current_sql = String::new();
        while !matches!(self.current(), Some(Token::End) | None) {
            match self.next() {
                Some(Token::Semicolon) => {
                    if !current_sql.is_empty() {
                        body.push(StoredProcStatement::RawSql(current_sql.trim().to_string()));
                        current_sql = String::new();
                    }
                }
                Some(Token::Identifier(sql)) => {
                    current_sql.push_str(&sql);
                    current_sql.push(' ');
                }
                Some(t) => {
                    current_sql.push_str(&t.to_string());
                    current_sql.push(' ');
                }
                None => return Err("Expected END".to_string()),
            }
        }
        self.expect(Token::End)?;

        if !current_sql.is_empty() {
            body.push(StoredProcStatement::RawSql(current_sql.trim().to_string()));
        }

        Ok(Statement::CreateProcedure(CreateProcedureStatement {
            name,
            params,
            body,
        }))
    }

    fn parse_create_trigger(&mut self) -> Result<Statement, String> {
        self.expect(Token::Trigger)?;

        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected trigger name, got {:?}", t)),
            None => return Err("Expected trigger name".to_string()),
        };

        let timing = match self.current() {
            Some(Token::Before) => {
                self.next();
                "BEFORE".to_string()
            }
            Some(Token::After) => {
                self.next();
                "AFTER".to_string()
            }
            Some(t) => return Err(format!("Expected BEFORE or AFTER, got {:?}", t)),
            None => return Err("Expected BEFORE or AFTER".to_string()),
        };

        let events = self.parse_trigger_events()?;

        self.expect(Token::On)?;

        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected table name, got {:?}", t)),
            None => return Err("Expected table name".to_string()),
        };

        self.expect(Token::ForEach)?;
        self.expect(Token::Each)?;
        self.expect(Token::Row)?;

        self.expect(Token::Begin)?;
        let mut body = String::new();
        while !matches!(self.current(), Some(Token::End) | None) {
            match self.next() {
                Some(Token::Semicolon) => {
                    body.push(';');
                    body.push(' ');
                }
                Some(Token::Identifier(sql)) => {
                    body.push_str(&sql);
                    body.push(' ');
                }
                Some(t) => {
                    body.push_str(&t.to_string());
                    body.push(' ');
                }
                None => return Err("Expected END".to_string()),
            }
        }
        self.expect(Token::End)?;

        Ok(Statement::CreateTrigger(CreateTriggerStatement {
            name,
            table,
            timing,
            events,
            body: body.trim().to_string(),
        }))
    }

    fn parse_trigger_events(&mut self) -> Result<Vec<String>, String> {
        let mut events = Vec::new();
        loop {
            match self.current() {
                Some(Token::Insert) => {
                    events.push("INSERT".to_string());
                    self.next();
                }
                Some(Token::Update) => {
                    events.push("UPDATE".to_string());
                    self.next();
                }
                Some(Token::Delete) => {
                    events.push("DELETE".to_string());
                    self.next();
                }
                Some(Token::Identifier(ref s))
                    if ["INSERT", "UPDATE", "DELETE"].contains(&s.to_uppercase().as_str()) =>
                {
                    events.push(s.to_uppercase().clone());
                    self.next();
                }
                Some(t) => {
                    if events.is_empty() {
                        return Err(format!("Expected INSERT, UPDATE, or DELETE, got {:?}", t));
                    }
                    break;
                }
                None => {
                    if events.is_empty() {
                        return Err("Expected INSERT, UPDATE, or DELETE".to_string());
                    }
                    break;
                }
            }
        }
        Ok(events)
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

        // Check for DISTINCT keyword (MySQL: SELECT DISTINCT col FROM t)
        let distinct = if matches!(self.current(), Some(Token::Distinct)) {
            self.next();
            true
        } else {
            false
        };

        let mut columns = Vec::new();
        let mut aggregates = Vec::new();

        loop {
            match self.current() {
                Some(Token::From) | Some(Token::Eof) => break,
                Some(Token::Star) => {
                    columns.push(SelectColumn {
                        name: "*".to_string(),
                        alias: None,
                        expression: None,
                    });
                    self.next();
                }
                // Handle aggregate functions: COUNT(*), SUM(col), etc.
                // Only treat as aggregate if followed by LParen
                Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
                | Some(Token::Max) => {
                    if matches!(self.peek(), Some(Token::LParen)) {
                        let func = self.parse_aggregate_function()?;
                        aggregates.push(func);
                        columns.push(SelectColumn {
                            name: format!("__agg_{}", aggregates.len()),
                            alias: None,
                            expression: None,
                        });
                    } else {
                        // Not followed by LParen - treat as identifier (column name)
                        // Convert keyword to identifier and process as column
                        let name = match self.current() {
                            Some(Token::Count) => "count",
                            Some(Token::Sum) => "sum",
                            Some(Token::Avg) => "avg",
                            Some(Token::Min) => "min",
                            Some(Token::Max) => "max",
                            _ => return Err("Expected column name".to_string()),
                        };
                        self.next(); // consume the keyword
                        columns.push(SelectColumn {
                            name: name.to_string(),
                            alias: None,
                            expression: Some(Expression::Identifier(name.to_string())),
                        });
                    }
                }
                Some(Token::LParen) => {
                    let expr = self.parse_expression()?;
                    self.expect(Token::RParen)?;
                    columns.push(SelectColumn {
                        name: format!("{:?}", expr),
                        alias: None,
                        expression: Some(expr),
                    });
                }
                // Handle NULL literal in SELECT
                Some(Token::Null) => {
                    columns.push(SelectColumn {
                        name: "NULL".to_string(),
                        alias: None,
                        expression: Some(Expression::Literal("NULL".to_string())),
                    });
                    self.next();
                }
                // Handle NumberLiteral in SELECT (e.g., SELECT 123, SELECT 3.14)
                Some(Token::NumberLiteral(ref n)) => {
                    columns.push(SelectColumn {
                        name: n.to_string(),
                        alias: None,
                        expression: Some(Expression::Literal(n.to_string())),
                    });
                    self.next();
                }
                // Handle StringLiteral in SELECT (e.g., SELECT 'hello')
                Some(Token::StringLiteral(s)) => {
                    columns.push(SelectColumn {
                        name: format!("'{}'", s),
                        alias: None,
                        expression: Some(Expression::Literal(format!("'{}'", s))),
                    });
                    self.next();
                }
                // Handle BooleanLiteral in SELECT (e.g., SELECT TRUE, FALSE)
                Some(Token::BooleanLiteral(b)) => {
                    let val = if *b { "TRUE" } else { "FALSE" };
                    columns.push(SelectColumn {
                        name: val.to_string(),
                        alias: None,
                        expression: Some(Expression::Literal(val.to_string())),
                    });
                    self.next();
                }
                Some(Token::Identifier(_)) => {
                    let start_position = self.position;
                    let (name, consumed, _is_expression) = match self.current().cloned() {
                        Some(Token::Identifier(name)) => {
                            if matches!(self.peek(), Some(Token::Dot)) {
                                let table = name.clone();
                                self.next();
                                self.expect(Token::Dot)?;
                                match self.current().cloned() {
                                    Some(Token::Identifier(col)) => {
                                        self.next();
                                        (format!("{}.{}", table, col), true, false)
                                    }
                                    Some(t) => {
                                        return Err(format!("Expected column name, got {:?}", t))
                                    }
                                    None => return Err("Expected column name".to_string()),
                                }
                            } else {
                                (name.clone(), false, false)
                            }
                        }
                        _ => return Err("Expected column name".to_string()),
                    };
                    let is_operator = if consumed {
                        matches!(self.current(), Some(Token::Plus))
                            || matches!(self.current(), Some(Token::Minus))
                            || matches!(self.current(), Some(Token::Star))
                            || matches!(self.current(), Some(Token::Slash))
                            || matches!(self.current(), Some(Token::Percent))
                            || matches!(self.current(), Some(Token::Equal))
                            || matches!(self.current(), Some(Token::NotEqual))
                            || matches!(self.current(), Some(Token::Greater))
                            || matches!(self.current(), Some(Token::Less))
                            || matches!(self.current(), Some(Token::GreaterEqual))
                            || matches!(self.current(), Some(Token::LessEqual))
                    } else {
                        matches!(self.peek(), Some(Token::Plus))
                            || matches!(self.peek(), Some(Token::Minus))
                            || matches!(self.peek(), Some(Token::Star))
                            || matches!(self.peek(), Some(Token::Slash))
                            || matches!(self.peek(), Some(Token::Percent))
                            || matches!(self.peek(), Some(Token::Equal))
                            || matches!(self.peek(), Some(Token::NotEqual))
                            || matches!(self.peek(), Some(Token::Greater))
                            || matches!(self.peek(), Some(Token::Less))
                            || matches!(self.peek(), Some(Token::GreaterEqual))
                            || matches!(self.peek(), Some(Token::LessEqual))
                    };

                    if is_operator {
                        if consumed {
                            let left = Expression::Identifier(name);
                            let op = match self.current() {
                                Some(Token::Plus) => "+",
                                Some(Token::Minus) => "-",
                                Some(Token::Star) => "*",
                                Some(Token::Slash) => "/",
                                Some(Token::Percent) => "%",
                                Some(Token::Equal) => "=",
                                Some(Token::NotEqual) => "!=",
                                Some(Token::Greater) => ">",
                                Some(Token::Less) => "<",
                                Some(Token::GreaterEqual) => ">=",
                                Some(Token::LessEqual) => "<=",
                                _ => return Err("Expected operator".to_string()),
                            };
                            self.next();
                            let right = self.parse_expression()?;
                            let expr = Expression::BinaryOp(
                                Box::new(left),
                                op.to_string(),
                                Box::new(right),
                            );
                            columns.push(SelectColumn {
                                name: format!("{:?}", expr),
                                alias: None,
                                expression: Some(expr),
                            });
                        } else {
                            self.position = start_position;
                            let expr = self.parse_expression()?;
                            columns.push(SelectColumn {
                                name: format!("{:?}", expr),
                                alias: None,
                                expression: Some(expr),
                            });
                        }
                    } else {
                        columns.push(SelectColumn {
                            name,
                            alias: None,
                            expression: None,
                        });
                        if !consumed {
                            self.next();
                        }
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

        // Handle SELECT without FROM (e.g., SELECT NULL, SELECT 1, SELECT 'hello')
        let table = match self.current() {
            Some(Token::From) => {
                self.next(); // consume FROM
                match self.next() {
                    Some(Token::Identifier(name)) => name,
                    Some(t) => return Err(format!("Expected table name, got {:?}", t)),
                    None => return Err("Expected table name".to_string()),
                }
            }
            Some(Token::Eof) | None => {
                // No FROM clause - this is a SELECT without table (e.g., SELECT 1+1)
                // Return an empty table name to indicate no table
                "".to_string()
            }
            Some(t) => return Err(format!("Expected FROM or end of query, got {:?}", t)),
        };

        // Check for table alias (e.g., `FROM users u`)
        if matches!(self.current(), Some(Token::Identifier(_))) {
            self.next(); // consume alias
        }

        // Check for JOIN
        let join_clause = if matches!(
            self.current(),
            Some(Token::Join)
                | Some(Token::Left)
                | Some(Token::Right)
                | Some(Token::Inner)
                | Some(Token::Full)
                | Some(Token::Cross)
        ) {
            Some(self.parse_join_clause()?)
        } else {
            None
        };

        let where_clause = if matches!(self.current(), Some(Token::Where)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse GROUP BY clause
        let group_by = if matches!(self.current(), Some(Token::Group)) {
            self.next();
            self.expect(Token::By)?;
            self.parse_expression_list()?
        } else {
            Vec::new()
        };

        // Parse HAVING clause
        let having = if matches!(self.current(), Some(Token::Having)) {
            self.next();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(SelectStatement {
            columns,
            table,
            where_clause,
            join_clause,
            aggregates,
            group_by,
            having,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            distinct,
        })
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>, String> {
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse_expression()?);
            if matches!(self.current(), Some(Token::Comma)) {
                self.next();
            } else {
                break;
            }
        }
        Ok(exprs)
    }

    fn parse_join_clause(&mut self) -> Result<JoinClause, String> {
        // Determine join type
        let join_type = match self.current() {
            Some(Token::Inner) => {
                self.next();
                JoinType::Inner
            }
            Some(Token::Left) => {
                self.next();
                JoinType::Left
            }
            Some(Token::Right) => {
                self.next();
                JoinType::Right
            }
            Some(Token::Full) => {
                self.next();
                if matches!(self.current(), Some(Token::Outer)) {
                    self.next();
                }
                JoinType::Full
            }
            Some(Token::Cross) => {
                self.next();
                JoinType::Cross
            }
            Some(Token::Join) => {
                self.next();
                JoinType::Inner
            }
            _ => return Err("Expected JOIN type".to_string()),
        };

        // If join type was LEFT/RIGHT/CROSS, we still need to consume the JOIN token
        if matches!(self.current(), Some(Token::Join)) {
            self.next();
        }

        // Parse joined table name
        let table = match self.current().cloned() {
            Some(Token::Identifier(name)) => {
                self.next();
                name
            }
            Some(t) => return Err(format!("Expected table name, got {:?}", t)),
            None => return Err("Expected table name".to_string()),
        };

        // Check for table alias (e.g., `JOIN orders o`)
        if matches!(self.current(), Some(Token::Identifier(_))) {
            self.next(); // consume alias
        }

        // Parse ON condition (optional for CROSS JOIN)
        let on_clause = if matches!(self.current(), Some(Token::On)) {
            self.next();
            self.parse_expression()?
        } else {
            Expression::Literal("true".to_string())
        };

        Ok(JoinClause {
            join_type,
            table,
            on_clause,
        })
    }

    fn parse_aggregate_function(&mut self) -> Result<AggregateCall, String> {
        let func = match self.current() {
            Some(Token::Count) => AggregateFunction::Count,
            Some(Token::Sum) => AggregateFunction::Sum,
            Some(Token::Avg) => AggregateFunction::Avg,
            Some(Token::Min) => AggregateFunction::Min,
            Some(Token::Max) => AggregateFunction::Max,
            _ => return Err("Expected aggregate function".to_string()),
        };
        self.next();

        self.expect(Token::LParen)?;

        let mut args = Vec::new();
        let mut distinct = false;

        if matches!(self.current(), Some(Token::Distinct)) {
            distinct = true;
            self.next();
        }

        // Handle COUNT(*) specially
        if matches!(self.current(), Some(Token::Star)) {
            self.next();
        } else if !matches!(self.current(), Some(Token::RParen)) {
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        let expr = if matches!(self.peek(), Some(Token::Dot)) {
                            let table = name.clone();
                            self.next();
                            self.expect(Token::Dot)?;
                            match self.current().cloned() {
                                Some(Token::Identifier(col)) => {
                                    self.next();
                                    Expression::Identifier(format!("{}.{}", table, col))
                                }
                                Some(t) => {
                                    return Err(format!("Expected column name, got {:?}", t))
                                }
                                None => return Err("Expected column name".to_string()),
                            }
                        } else {
                            self.next();
                            Expression::Identifier(name)
                        };
                        args.push(expr);
                    }
                    Some(Token::NumberLiteral(n)) => {
                        args.push(Expression::Literal(n.clone()));
                        self.next();
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }

        self.expect(Token::RParen)?;

        Ok(AggregateCall {
            func,
            args,
            distinct,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        // Check for REPLACE INTO (MySQL compatibility) - consume Replace token if present
        let is_replace = if matches!(self.current(), Some(Token::Replace)) {
            self.next(); // consume Replace
            true
        } else {
            self.expect(Token::Insert)?;
            false
        };

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
                        Some(Token::Comma) => {
                            self.next();
                        }
                        _ => {
                            let expr = self.parse_expression()?;
                            row.push(expr);
                        }
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
            is_replace,
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

            // Parse value - use parse_expression to support binary operations
            let value = self.parse_expression()?;

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
        let left = self.parse_additive_expression()?;

        if matches!(self.current(), Some(Token::In)) {
            self.next();
            self.expect(Token::LParen)?;
            // Check if it's a subquery (SELECT ...) or a value list (1, 2, 3)
            if matches!(self.current(), Some(Token::Select)) {
                let subquery = self.parse_select_statement()?;
                self.expect(Token::RParen)?;
                return Ok(Expression::In(Box::new(left), Box::new(subquery)));
            } else {
                // Parse value list: IN (1, 2, 3)
                let mut values = Vec::new();
                loop {
                    values.push(self.parse_expression()?);
                    if matches!(self.current(), Some(Token::Comma)) {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                return Ok(Expression::InList(Box::new(left), values));
            }
        }

        if matches!(self.current(), Some(Token::Not)) {
            self.next();
            if matches!(self.current(), Some(Token::In)) {
                self.next();
                self.expect(Token::LParen)?;
                // Check if it's a subquery or value list
                if matches!(self.current(), Some(Token::Select)) {
                    let subquery = self.parse_select_statement()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expression::NotIn(Box::new(left), Box::new(subquery)));
                } else {
                    // Parse value list: NOT IN (1, 2, 3)
                    let mut values = Vec::new();
                    loop {
                        values.push(self.parse_expression()?);
                        if matches!(self.current(), Some(Token::Comma)) {
                            self.next();
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    return Ok(Expression::NotInList(Box::new(left), values));
                }
            }
            return Err("NOT must be followed by IN or EXISTS".to_string());
        }

        // IS NULL / IS NOT NULL
        if matches!(self.current(), Some(Token::Is)) {
            self.next();
            if matches!(self.current(), Some(Token::Not)) {
                self.next();
                if matches!(self.current(), Some(Token::Null)) {
                    self.next();
                    return Ok(Expression::IsNotNull(Box::new(left)));
                }
                return Err("IS NOT must be followed by NULL".to_string());
            }
            if matches!(self.current(), Some(Token::Null)) {
                self.next();
                return Ok(Expression::IsNull(Box::new(left)));
            }
            return Err("IS must be followed by NULL or NOT NULL".to_string());
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

    fn parse_additive_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(Token::Plus) | Some(Token::Minus) = self.current() {
            let op = match self.current() {
                Some(Token::Plus) => "+",
                Some(Token::Minus) => "-",
                _ => break,
            };
            self.next();
            let right = self.parse_multiplicative_expression()?;
            left = Expression::BinaryOp(Box::new(left), op.to_string(), Box::new(right));
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_primary_expression()?;

        while let Some(Token::Star) | Some(Token::Slash) = self.current() {
            let op = match self.current() {
                Some(Token::Star) => "*",
                Some(Token::Slash) => "/",
                _ => break,
            };
            self.next();
            let right = self.parse_primary_expression()?;
            left = Expression::BinaryOp(Box::new(left), op.to_string(), Box::new(right));
        }

        Ok(left)
    }

    /// Parse primary expression (identifier, literal, or parenthesized)
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.current() {
            Some(Token::Identifier(_)) => {
                let name = match self.current() {
                    Some(Token::Identifier(n)) => n.clone(),
                    _ => return Err("Expected identifier".to_string()),
                };
                self.next();
                if matches!(self.current(), Some(Token::Dot)) {
                    self.next();
                    match self.current() {
                        Some(Token::Identifier(col)) => {
                            let col_name = col.clone();
                            self.next();
                            Ok(Expression::Identifier(format!("{}.{}", name, col_name)))
                        }
                        Some(t) => Err(format!("Expected column name after dot, got {:?}", t)),
                        None => Err("Expected column name after dot".to_string()),
                    }
                } else {
                    Ok(Expression::Identifier(name))
                }
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
            // Support aggregate functions in expressions (for HAVING clause)
            Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
            | Some(Token::Max) => {
                let agg = self.parse_aggregate_function()?;
                Ok(Expression::Aggregate(agg))
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
        // Check if we need to consume Token::Create (may have been consumed by parse_statement)
        if matches!(self.current(), Some(Token::Create)) {
            self.next(); // consume CREATE if not already consumed
        }
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
        let mut auto_increment = false;
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
                    let ref_columns = if matches!(self.current(), Some(Token::LParen)) {
                        self.next();
                        self.parse_column_list()?
                    } else {
                        vec![]
                    };
                    let (on_delete, on_update) = self.parse_referential_actions()?;
                    references = Some(ForeignKeyRef {
                        columns: vec![name.clone()],
                        referenced_table: ref_table,
                        referenced_columns: ref_columns,
                        on_delete,
                        on_update,
                    });
                }
                Some(Token::AutoIncrement) => {
                    self.next();
                    auto_increment = true;
                }
                _ => break,
            }
        }

        Ok(ColumnDefinition {
            name,
            data_type,
            nullable,
            primary_key,
            auto_increment,
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
        let referenced_columns = if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            self.parse_column_list()?
        } else {
            vec![]
        };
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
        while let Some(Token::On) = self.current() {
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

    fn parse_truncate(&mut self) -> Result<Statement, String> {
        self.expect(Token::Truncate)?;
        self.expect(Token::Table)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        Ok(Statement::Truncate(TruncateStatement { name }))
    }

    fn parse_show(&mut self) -> Result<Statement, String> {
        self.expect(Token::Show)?;

        match self.current() {
            Some(Token::Identifier(ref ident)) if ident.to_uppercase() == "TABLES" => {
                self.next();
                Ok(Statement::Show(ShowStatement::Tables))
            }
            Some(Token::Identifier(ref ident)) if ident.to_uppercase() == "COLUMNS" => {
                self.next();
                self.expect(Token::From)?;
                let table = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected table name".to_string()),
                };
                let pattern = if matches!(self.current(), Some(Token::Identifier(ref ident)) if ident.to_uppercase() == "LIKE")
                {
                    self.next();
                    match self.next() {
                        Some(Token::StringLiteral(p)) => Some(p),
                        _ => return Err("Expected pattern string".to_string()),
                    }
                } else {
                    None
                };
                Ok(Statement::Show(ShowStatement::Columns { table, pattern }))
            }
            Some(Token::Identifier(ref ident)) if ident.to_uppercase() == "INDEX" => {
                self.next();
                self.expect(Token::From)?;
                let table = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected table name".to_string()),
                };
                Ok(Statement::Show(ShowStatement::Index { table }))
            }
            Some(Token::Index) => {
                self.next();
                self.expect(Token::From)?;
                let table = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected table name".to_string()),
                };
                Ok(Statement::Show(ShowStatement::Index { table }))
            }
            Some(Token::Identifier(ref ident)) if ident.to_uppercase() == "GRANTS" => {
                self.next();
                let user = if matches!(self.current(), Some(Token::ForEach)) {
                    self.next();
                    match self.next() {
                        Some(Token::StringLiteral(u)) => Some(u),
                        Some(Token::Identifier(u)) => Some(u),
                        _ => return Err("Expected user string".to_string()),
                    }
                } else {
                    None
                };
                Ok(Statement::Show(ShowStatement::Grants { user }))
            }
            Some(t) => Err(format!("Unexpected token after SHOW: {:?}", t)),
            None => Err("Unexpected end of input after SHOW".to_string()),
        }
    }

    fn parse_describe(&mut self) -> Result<Statement, String> {
        self.expect(Token::Describe)?;
        let table = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };
        Ok(Statement::Describe(DescribeStatement { table }))
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

    fn parse_grant(&mut self) -> Result<Statement, String> {
        self.expect(Token::Grant)?;

        let mut privileges = Vec::new();
        loop {
            match self.current() {
                Some(Token::Select) => {
                    privileges.push(Privilege::Select);
                    self.next();
                }
                Some(Token::Insert) => {
                    privileges.push(Privilege::Insert);
                    self.next();
                }
                Some(Token::Update) => {
                    privileges.push(Privilege::Update);
                    self.next();
                }
                Some(Token::Delete) => {
                    privileges.push(Privilege::Delete);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "READ" => {
                    privileges.push(Privilege::Read);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "WRITE" => {
                    privileges.push(Privilege::Write);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "ALL" => {
                    privileges.push(Privilege::All);
                    self.next();
                    break;
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "EXECUTE" => {
                    privileges.push(Privilege::Execute);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "USAGE" => {
                    privileges.push(Privilege::Usage);
                    self.next();
                }
                _ => break,
            }

            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        let mut columns = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        columns.push(name.clone());
                        self.next();
                    }
                    _ => return Err("Expected column name".to_string()),
                }
                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.next();
            }
            self.expect(Token::RParen)?;
        }

        self.expect(Token::On)?;

        let object_type = match self.current() {
            Some(Token::Identifier(name)) if name.to_uppercase() == "TABLE" => {
                self.next();
                ObjectType::Table
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "DATABASE" => {
                self.next();
                ObjectType::Database
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "PROCEDURE" => {
                self.next();
                ObjectType::Procedure
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "FUNCTION" => {
                self.next();
                ObjectType::Function
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "COLUMN" => {
                self.next();
                ObjectType::Column
            }
            _ => ObjectType::Table,
        };

        let object_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected object name, got {:?}", t)),
            None => return Err("Expected object name".to_string()),
        };

        self.expect(Token::To)?;

        let mut recipients = Vec::new();
        loop {
            match self.next() {
                Some(Token::Identifier(name)) => {
                    recipients.push(name);
                }
                Some(t) => return Err(format!("Expected recipient user, got {:?}", t)),
                None => return Err("Expected recipient user".to_string()),
            }
            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        let with_grant_option = if let Some(Token::Identifier(name)) = self.current() {
            if name.to_uppercase() == "WITH" {
                self.next();
                if let Some(Token::Identifier(grant_str)) = self.current() {
                    if grant_str.to_uppercase() == "GRANT" {
                        self.next();
                        if let Some(Token::Identifier(option_str)) = self.current() {
                            if option_str.to_uppercase() == "OPTION" {
                                self.next();
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        Ok(Statement::Grant(GrantStatement {
            privileges,
            columns,
            object_type,
            object_name,
            recipients,
            with_grant_option,
        }))
    }

    fn parse_revoke(&mut self) -> Result<Statement, String> {
        self.expect(Token::Revoke)?;

        let grant_option_for = if let Some(Token::Identifier(name)) = self.current() {
            if name.to_uppercase() == "GRANT" {
                self.next();
                if let Some(Token::Identifier(option_str)) = self.current() {
                    if option_str.to_uppercase() == "OPTION" {
                        self.next();
                        if let Some(Token::Identifier(for_str)) = self.current() {
                            if for_str.to_uppercase() == "FOR" {
                                self.next();
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        let mut privileges = Vec::new();
        loop {
            match self.current() {
                Some(Token::Select) => {
                    privileges.push(Privilege::Select);
                    self.next();
                }
                Some(Token::Insert) => {
                    privileges.push(Privilege::Insert);
                    self.next();
                }
                Some(Token::Update) => {
                    privileges.push(Privilege::Update);
                    self.next();
                }
                Some(Token::Delete) => {
                    privileges.push(Privilege::Delete);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "READ" => {
                    privileges.push(Privilege::Read);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "WRITE" => {
                    privileges.push(Privilege::Write);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "ALL" => {
                    privileges.push(Privilege::All);
                    self.next();
                    break;
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "EXECUTE" => {
                    privileges.push(Privilege::Execute);
                    self.next();
                }
                Some(Token::Identifier(name)) if name.to_uppercase() == "USAGE" => {
                    privileges.push(Privilege::Usage);
                    self.next();
                }
                _ => break,
            }

            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        let mut columns = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            loop {
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        columns.push(name.clone());
                        self.next();
                    }
                    _ => return Err("Expected column name".to_string()),
                }
                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.next();
            }
            self.expect(Token::RParen)?;
        }

        self.expect(Token::On)?;

        let object_type = match self.current() {
            Some(Token::Identifier(name)) if name.to_uppercase() == "TABLE" => {
                self.next();
                ObjectType::Table
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "DATABASE" => {
                self.next();
                ObjectType::Database
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "PROCEDURE" => {
                self.next();
                ObjectType::Procedure
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "FUNCTION" => {
                self.next();
                ObjectType::Function
            }
            Some(Token::Identifier(name)) if name.to_uppercase() == "COLUMN" => {
                self.next();
                ObjectType::Column
            }
            _ => ObjectType::Table,
        };

        let object_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected object name, got {:?}", t)),
            None => return Err("Expected object name".to_string()),
        };

        self.expect(Token::From)?;

        let mut from_users = Vec::new();
        loop {
            match self.next() {
                Some(Token::Identifier(name)) => {
                    from_users.push(name);
                }
                Some(t) => return Err(format!("Expected user, got {:?}", t)),
                None => return Err("Expected user".to_string()),
            }
            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        Ok(Statement::Revoke(RevokeStatement {
            privileges,
            columns,
            object_type,
            object_name,
            from_users,
            grant_option_for,
        }))
    }

    fn parse_call(&mut self) -> Result<Statement, String> {
        self.expect(Token::Call)?;

        let procedure_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            Some(t) => return Err(format!("Expected procedure name, got {:?}", t)),
            None => return Err("Expected procedure name".to_string()),
        };

        let mut args = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next();
            while !matches!(self.current(), Some(Token::RParen) | None) {
                match self.current() {
                    Some(
                        Token::Identifier(_)
                        | Token::StringLiteral(_)
                        | Token::NumberLiteral(_)
                        | Token::Null,
                    ) => {
                        let arg = match self.next() {
                            Some(Token::Identifier(s)) => s,
                            Some(Token::StringLiteral(s)) => s,
                            Some(Token::NumberLiteral(s)) => s,
                            Some(Token::Null) => "NULL".to_string(),
                            Some(t) => return Err(format!("Expected argument, got {:?}", t)),
                            None => return Err("Unexpected end of input".to_string()),
                        };
                        args.push(arg);
                    }
                    Some(Token::Comma) => {
                        self.next();
                    }
                    Some(t) => return Err(format!("Unexpected token in argument list: {:?}", t)),
                    None => return Err("Unexpected end of input".to_string()),
                }
            }
            self.expect(Token::RParen)?;
        }

        Ok(Statement::Call(CallStatement {
            procedure_name,
            args,
        }))
    }

    fn parse_alter_table(&mut self) -> Result<Statement, String> {
        self.expect(Token::Alter)?;
        self.expect(Token::Table)?;

        let table_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        match self.current() {
            Some(Token::Add) => {
                self.next();
                if matches!(self.current(), Some(Token::Column)) {
                    self.next();
                }
                let col_name = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected column name".to_string()),
                };

                let data_type = match self.next() {
                    Some(Token::Identifier(typename)) => typename,
                    Some(Token::Integer) => "INTEGER".to_string(),
                    Some(Token::Text) => "TEXT".to_string(),
                    Some(Token::Float) => "FLOAT".to_string(),
                    Some(Token::Boolean) => "BOOLEAN".to_string(),
                    _ => return Err("Expected data type".to_string()),
                };

                let nullable = true;
                let default_value = None;

                Ok(Statement::AlterTable(AlterTableStatement {
                    table_name,
                    operation: AlterTableOperation::AddColumn {
                        name: col_name,
                        data_type,
                        nullable,
                        default_value,
                    },
                }))
            }
            Some(Token::Rename) => {
                self.next();
                self.expect(Token::To)?;
                let new_name = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected new table name".to_string()),
                };
                Ok(Statement::AlterTable(AlterTableStatement {
                    table_name,
                    operation: AlterTableOperation::RenameTo { new_name },
                }))
            }
            _ => Err("Expected ADD or RENAME".to_string()),
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
    fn test_parse_qualified_column_names() {
        let sql = "SELECT u.name FROM t";
        let result = parse(sql);
        assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "t");
                assert_eq!(s.columns.len(), 1);
                assert_eq!(s.columns[0].name, "u.name");
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
    fn test_parse_replace_into() {
        let result = parse("REPLACE INTO users VALUES (1, 'Alice')");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert!(i.is_replace, "REPLACE INTO should set is_replace to true");
                assert_eq!(i.values.len(), 1); // 1 row
                assert_eq!(i.values[0].len(), 2); // 2 values per row
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_replace_into_with_columns() {
        let result = parse("REPLACE INTO users (id, name) VALUES (1, 'Alice')");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Insert(i) => {
                assert_eq!(i.table, "users");
                assert!(i.is_replace, "REPLACE INTO should set is_replace to true");
                assert_eq!(i.columns, vec!["id", "name"]);
                assert_eq!(i.values.len(), 1); // 1 row
                assert_eq!(i.values[0].len(), 2); // 2 values
            }
            _ => panic!("Expected INSERT statement"),
        }
    }

    #[test]
    fn test_parse_inner_join() {
        let result =
            parse("SELECT name, amount FROM users JOIN orders ON users.id = orders.user_id");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert!(s.join_clause.is_some());
                let join = s.join_clause.unwrap();
                assert_eq!(join.table, "orders");
                assert_eq!(join.join_type, JoinType::Inner);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_left_join() {
        let result =
            parse("SELECT name, amount FROM users LEFT JOIN orders ON users.id = orders.user_id");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert!(s.join_clause.is_some());
                let join = s.join_clause.unwrap();
                assert_eq!(join.table, "orders");
                assert_eq!(join.join_type, JoinType::Left);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_full_join() {
        let result = parse("SELECT * FROM t1 FULL JOIN t2 ON t1.id = t2.id");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "t1");
                assert!(s.join_clause.is_some());
                let join = s.join_clause.unwrap();
                assert_eq!(join.table, "t2");
                assert_eq!(join.join_type, JoinType::Full);
            }
            _ => panic!("Expected SELECT statement"),
        }

        let result2 = parse("SELECT * FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id");
        assert!(result2.is_ok(), "Parse failed: {:?}", result2);
        match result2.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "t1");
                assert!(s.join_clause.is_some());
                let join = s.join_clause.unwrap();
                assert_eq!(join.table, "t2");
                assert_eq!(join.join_type, JoinType::Full);
            }
            _ => panic!("Expected SELECT statement"),
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
    #[ignore]
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

    #[test]
    fn test_parse_create_with_auto_increment() {
        let result = parse("CREATE TABLE t (id INT AUTO_INCREMENT PRIMARY KEY)");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::CreateTable(c) => {
                assert_eq!(c.name, "t");
                assert_eq!(c.columns.len(), 1);
                assert!(c.columns[0].auto_increment);
                assert!(c.columns[0].primary_key);
                assert!(!c.columns[0].nullable);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_aggregate_count() {
        let result = parse("SELECT COUNT(*) FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.columns.len(), 1);
                assert_eq!(s.aggregates.len(), 1);
                assert_eq!(s.aggregates[0].func, AggregateFunction::Count);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_aggregate_sum() {
        let result = parse("SELECT SUM(amount) FROM orders");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "orders");
                assert_eq!(s.aggregates.len(), 1);
                assert_eq!(s.aggregates[0].func, AggregateFunction::Sum);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let result = parse("DELETE FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
                assert!(d.where_clause.is_none());
            }
            _ => panic!("Expected DELETE statement"),
        }
    }

    #[test]
    fn test_parse_delete_with_where() {
        let result = parse("DELETE FROM users WHERE id = 1");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Delete(d) => {
                assert_eq!(d.table, "users");
                assert!(d.where_clause.is_some());
            }
            _ => panic!("Expected DELETE statement"),
        }
    }

    #[test]
    fn test_parse_analyze() {
        let result = parse("ANALYZE users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Analyze(a) => {
                assert_eq!(a.table_name, Some("users".to_string()));
            }
            _ => panic!("Expected ANALYZE statement"),
        }
    }

    #[test]
    fn test_parse_show_tables() {
        let result = parse("SHOW TABLES");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Show(ShowStatement::Tables) => {}
            _ => panic!("Expected SHOW TABLES statement"),
        }
    }

    #[test]
    fn test_parse_show_columns() {
        let result = parse("SHOW COLUMNS FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Show(ShowStatement::Columns { table, pattern }) => {
                assert_eq!(table, "users");
                assert!(pattern.is_none());
            }
            _ => panic!("Expected SHOW COLUMNS FROM users statement"),
        }
    }

    #[test]
    fn test_parse_show_columns_with_like() {
        let result = parse("SHOW COLUMNS FROM users LIKE '%name%'");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Show(ShowStatement::Columns { table, pattern }) => {
                assert_eq!(table, "users");
                assert_eq!(pattern, Some("%name%".to_string()));
            }
            _ => panic!("Expected SHOW COLUMNS FROM users LIKE '%name%' statement"),
        }
    }

    #[test]
    fn test_parse_show_index() {
        let result = parse("SHOW INDEX FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Show(ShowStatement::Index { table }) => {
                assert_eq!(table, "users");
            }
            _ => panic!("Expected SHOW INDEX FROM users statement"),
        }
    }

    #[test]
    fn test_parse_describe() {
        let result = parse("DESCRIBE users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Describe(DescribeStatement { table }) => {
                assert_eq!(table, "users");
            }
            _ => panic!("Expected DESCRIBE users statement"),
        }
    }

    #[test]
    fn test_parse_desc_alias() {
        let result = parse("DESC users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Describe(DescribeStatement { table }) => {
                assert_eq!(table, "users");
            }
            _ => panic!("Expected DESC users statement"),
        }
    }

    #[test]
    fn test_parse_alter_table_add_column() {
        let result = parse("ALTER TABLE users ADD COLUMN age INTEGER");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::AlterTable(a) => {
                assert_eq!(a.table_name, "users");
                match a.operation {
                    AlterTableOperation::AddColumn {
                        name, data_type, ..
                    } => {
                        assert_eq!(name, "age");
                        assert_eq!(data_type, "INTEGER");
                    }
                    _ => panic!("Expected AddColumn operation"),
                }
            }
            _ => panic!("Expected ALTER TABLE statement"),
        }
    }

    #[test]
    fn test_parse_alter_table_rename_to() {
        let result = parse("ALTER TABLE users RENAME TO old_users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::AlterTable(a) => {
                assert_eq!(a.table_name, "users");
                match a.operation {
                    AlterTableOperation::RenameTo { new_name } => {
                        assert_eq!(new_name, "old_users");
                    }
                    _ => panic!("Expected RenameTo operation"),
                }
            }
            _ => panic!("Expected ALTER TABLE statement"),
        }
    }

    #[test]
    fn test_parse_right_join() {
        let result = parse("SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert!(s.join_clause.is_some());
                assert_eq!(s.join_clause.unwrap().join_type, JoinType::Right);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_call() {
        let result = parse("CALL test_proc()");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Call(c) => {
                assert_eq!(c.procedure_name, "test_proc");
                assert!(c.args.is_empty());
            }
            _ => panic!("Expected CALL statement"),
        }
    }

    #[test]
    fn test_parse_call_with_args() {
        let result = parse("CALL test_proc(1, 'hello', var1)");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Call(c) => {
                assert_eq!(c.procedure_name, "test_proc");
                assert_eq!(c.args.len(), 3);
                assert_eq!(c.args[0], "1");
                assert_eq!(c.args[1], "hello");
                assert_eq!(c.args[2], "var1");
            }
            _ => panic!("Expected CALL statement"),
        }
    }

    #[test]
    fn test_parse_comparison_expression() {
        let result = parse("SELECT * FROM t WHERE a > b AND c < d");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert!(s.where_clause.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_like_expression() {
        let result = parse("SELECT * FROM t WHERE name LIKE '%test%'");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert!(s.where_clause.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_aggregate_avg() {
        let result = parse("SELECT AVG(price) FROM orders");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "orders");
                assert_eq!(s.aggregates.len(), 1);
                assert_eq!(s.aggregates[0].func, AggregateFunction::Avg);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_aggregate_min_max() {
        let result = parse("SELECT MIN(id), MAX(id) FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.table, "users");
                assert_eq!(s.aggregates.len(), 2);
                assert_eq!(s.aggregates[0].func, AggregateFunction::Min);
                assert_eq!(s.aggregates[1].func, AggregateFunction::Max);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
}

#[test]
fn test_debug_having() {
    let sql =
        "SELECT region, SUM(amount) FROM sales_summary GROUP BY region HAVING SUM(amount) > 150";
    match parse(sql) {
        Ok(stmt) => {
            println!("OK: {:#?}", stmt);
            if let Statement::Select(s) = stmt {
                println!("having = {:?}", s.having);
            }
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }

    #[test]
    fn test_parse_binary_expression_subtraction() {
        let result = parse("SELECT price - discount FROM orders");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_multiplication() {
        let result = parse("SELECT quantity * price FROM orders");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_division() {
        let result = parse("SELECT total / cnt FROM stats");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_modulo() {
        let result = parse("SELECT total % discount FROM orders");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_not_equal() {
        let result = parse("SELECT a != b FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_less_equal() {
        let result = parse("SELECT a <= b FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_greater_equal() {
        let result = parse("SELECT a >= b FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_complex() {
        let result = parse("SELECT a + b * c - d / e FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_with_literal() {
        let result = parse("SELECT id + 1 FROM users");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_multiple_columns() {
        let result = parse("SELECT a + b, c - d, e * f FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 3);
                assert!(s.columns[0].expression.is_some());
                assert!(s.columns[1].expression.is_some());
                assert!(s.columns[2].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_mixed_with_identifier() {
        let result = parse("SELECT a + b, name, c * d FROM t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 3);
                assert!(s.columns[0].expression.is_some());
                assert!(s.columns[1].expression.is_none());
                assert!(s.columns[2].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression_with_table_prefix() {
        let result = parse("SELECT t.a + t.b FROM table_name t");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Select(s) => {
                assert_eq!(s.columns.len(), 1);
                assert!(s.columns[0].expression.is_some());
            }
            _ => panic!("Expected SELECT statement"),
        }
    }

    #[test]
    fn test_parse_create_trigger() {
        let sql = "CREATE TRIGGER test_trigger BEFORE INSERT ON users FOR EACH ROW BEGIN SET NEW.name = 'triggered'; END";
        let result = parse(sql);
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::CreateTrigger(t) => {
                assert_eq!(t.name, "test_trigger");
                assert_eq!(t.timing, "BEFORE");
                assert_eq!(t.table, "users");
                assert_eq!(t.events.len(), 1);
                assert_eq!(t.events[0], "INSERT");
                assert!(t.body.contains("SET NEW.name"));
            }
            _ => panic!("Expected CREATE TRIGGER statement"),
        }
    }

    #[test]
    fn test_parse_create_trigger_after_update() {
        let sql = "CREATE TRIGGER update_trigger AFTER UPDATE ON orders FOR EACH ROW BEGIN DELETE FROM log; END";
        let result = parse(sql);
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::CreateTrigger(t) => {
                assert_eq!(t.name, "update_trigger");
                assert_eq!(t.timing, "AFTER");
                assert_eq!(t.table, "orders");
                assert_eq!(t.events.len(), 1);
                assert_eq!(t.events[0], "UPDATE");
            }
            _ => panic!("Expected CREATE TRIGGER statement"),
        }
    }

    #[test]
    fn test_parse_begin() {
        let result = parse("BEGIN");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(!work);
                assert!(isolation_level.is_none());
            }
            _ => panic!("Expected BEGIN statement"),
        }
    }

    #[test]
    fn test_parse_begin_work() {
        let result = parse("BEGIN WORK");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(work);
                assert!(isolation_level.is_none());
            }
            _ => panic!("Expected BEGIN WORK statement"),
        }
    }

    #[test]
    fn test_parse_begin_serializable() {
        let result = parse("BEGIN SERIALIZABLE");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(!work);
                assert_eq!(isolation_level, Some(IsolationLevel::Serializable));
            }
            _ => panic!("Expected BEGIN SERIALIZABLE statement"),
        }
    }

    #[test]
    fn test_parse_begin_isolation_level() {
        let result = parse("BEGIN ISOLATION LEVEL SERIALIZABLE");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(!work);
                assert_eq!(isolation_level, Some(IsolationLevel::Serializable));
            }
            _ => panic!("Expected BEGIN ISOLATION LEVEL SERIALIZABLE statement"),
        }
    }

    #[test]
    fn test_parse_commit() {
        let result = parse("COMMIT");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Commit { work }) => {
                assert!(!work);
            }
            _ => panic!("Expected COMMIT statement"),
        }
    }

    #[test]
    fn test_parse_commit_work() {
        let result = parse("COMMIT WORK");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Commit { work }) => {
                assert!(work);
            }
            _ => panic!("Expected COMMIT WORK statement"),
        }
    }

    #[test]
    fn test_parse_rollback() {
        let result = parse("ROLLBACK");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Rollback { work }) => {
                assert!(!work);
            }
            _ => panic!("Expected ROLLBACK statement"),
        }
    }

    #[test]
    fn test_parse_rollback_work() {
        let result = parse("ROLLBACK WORK");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Rollback { work }) => {
                assert!(work);
            }
            _ => panic!("Expected ROLLBACK WORK statement"),
        }
    }

    #[test]
    fn test_parse_start_transaction() {
        let result = parse("START TRANSACTION");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::StartTransaction { isolation_level }) => {
                assert!(isolation_level.is_none());
            }
            _ => panic!("Expected START TRANSACTION statement"),
        }
    }

    #[test]
    fn test_parse_start_transaction_serializable() {
        let result = parse("START TRANSACTION ISOLATION LEVEL SERIALIZABLE");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::StartTransaction { isolation_level }) => {
                assert_eq!(isolation_level, Some(IsolationLevel::Serializable));
            }
            _ => panic!("Expected START TRANSACTION ISOLATION LEVEL SERIALIZABLE statement"),
        }
    }

    #[test]
    fn test_parse_set_transaction() {
        let result = parse("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::SetTransaction { isolation_level }) => {
                assert_eq!(isolation_level, IsolationLevel::Serializable);
            }
            _ => panic!("Expected SET TRANSACTION statement"),
        }
    }

    #[test]
    fn test_parse_set_transaction_read_committed() {
        let result = parse("SET TRANSACTION ISOLATION LEVEL READ COMMITTED");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::SetTransaction { isolation_level }) => {
                assert_eq!(isolation_level, IsolationLevel::ReadCommitted);
            }
            _ => panic!("Expected SET TRANSACTION ISOLATION LEVEL READ COMMITTED statement"),
        }
    }

    #[test]
    fn test_parse_set_transaction_read_uncommitted() {
        let result = parse("SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::SetTransaction { isolation_level }) => {
                assert_eq!(isolation_level, IsolationLevel::ReadUncommitted);
            }
            _ => panic!("Expected SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED statement"),
        }
    }

    #[test]
    fn test_parse_begin_read_committed() {
        let result = parse("BEGIN ISOLATION LEVEL READ COMMITTED");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(!work);
                assert_eq!(isolation_level, Some(IsolationLevel::ReadCommitted));
            }
            _ => panic!("Expected BEGIN ISOLATION LEVEL READ COMMITTED statement"),
        }
    }

    #[test]
    fn test_parse_begin_repeatable_read() {
        let result = parse("BEGIN ISOLATION LEVEL REPEATABLE READ");
        assert!(result.is_ok(), "Parse failed: {:?}", result);
        match result.unwrap() {
            Statement::Transaction(TransactionStatement::Begin {
                work,
                isolation_level,
            }) => {
                assert!(!work);
                assert_eq!(isolation_level, Some(IsolationLevel::SnapshotIsolation));
            }
            _ => panic!("Expected BEGIN ISOLATION LEVEL REPEATABLE READ statement"),
        }
    }
}

#[test]
fn test_parse_is_null_expression() {
    let result = parse("SELECT * FROM t WHERE col IS NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert!(s.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_is_not_null_expression() {
    let result = parse("SELECT * FROM t WHERE col IS NOT NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert!(s.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_null_literal_is_null() {
    let result = parse("SELECT * FROM t WHERE NULL IS NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_null_literal_is_not_null() {
    let result = parse("SELECT * FROM t WHERE NULL IS NOT NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_and_with_null_in_where() {
    let result = parse("SELECT * FROM t WHERE col1 IS NULL AND col2 IS NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_or_with_null_in_where() {
    let result = parse("SELECT * FROM t WHERE col1 IS NULL OR col2 IS NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_is_not_null_in_where() {
    let result = parse("SELECT * FROM t WHERE col IS NOT NULL AND other_col IS NOT NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_complex_three_valued_expression() {
    let result = parse("SELECT * FROM t WHERE col1 IS NULL AND col2 IS NOT NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_is_null_in_join_condition() {
    let result = parse("SELECT * FROM t1 JOIN t2 ON t1.id IS NULL");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_cross_join() {
    let result = parse("SELECT * FROM t1 CROSS JOIN t2");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "t1");
            assert!(s.join_clause.is_some());
            let join = s.join_clause.unwrap();
            assert_eq!(join.table, "t2");
            assert_eq!(join.join_type, JoinType::Cross);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_join_on_multiple_conditions() {
    let result = parse("SELECT * FROM t1 JOIN t2 ON t1.id = t2.id AND t1.type = t2.type");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert!(s.join_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_join_with_table_alias() {
    let result = parse("SELECT * FROM users u JOIN orders o ON u.id = o.user_id");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "users");
            assert!(s.join_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_join_without_on_clause_implicit_inner() {
    let result = parse("SELECT * FROM t1 JOIN t2");
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "t1");
            assert!(s.join_clause.is_some());
            let join = s.join_clause.unwrap();
            assert_eq!(join.join_type, JoinType::Inner);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_debug_fk() {
    let sql = "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount INTEGER)";
    match parse(sql) {
        Ok(stmt) => println!("OK: {:#?}", stmt),
        Err(e) => println!("ERROR: {}", e),
    }
}

#[test]
fn test_debug_refs() {
    let sql = "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), amount INTEGER)";
    match parse(sql) {
        Ok(stmt) => println!("OK: {:#?}", stmt),
        Err(e) => println!("ERROR: {}", e),
    }
}

#[test]
fn test_debug_refs2() {
    let sql = "CREATE TABLE orders (user_id INTEGER REFERENCES users(id))";
    match parse(sql) {
        Ok(stmt) => println!("OK: {:#?}", stmt),
        Err(e) => println!("ERROR: {}", e),
    }
}

#[test]
fn test_debug_exact() {
    let sql = "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), amount INTEGER)";
    match parse(sql) {
        Ok(stmt) => println!("OK: {:#?}", stmt),
        Err(e) => println!("ERROR: {}", e),
    }
}

#[test]
fn test_debug_cascade() {
    // This is EXACTLY what's in cascade.sql
    let sql1 = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)";
    println!("Test 1: {:?}", parse(sql1));

    let sql2 = "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), amount INTEGER)";
    println!("Test 2: {:?}", parse(sql2));

    let sql3 = "CREATE INDEX idx_orders_user_id ON orders(user_id)";
    println!("Test 3: {:?}", parse(sql3));
}

#[test]
fn test_debug_idx() {
    use crate::{lexer::Lexer, parse};

    let sql = "CREATE INDEX idx_orders_user_id ON orders(user_id)";
    println!("SQL: [{}]", sql);
    let tokens = Lexer::new(sql).tokenize();
    println!("Tokens: {:?}", tokens);

    match parse(sql) {
        Ok(stmt) => println!("OK: {:#?}", stmt),
        Err(e) => println!("ERROR: {}", e),
    }
}
