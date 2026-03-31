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
#[allow(clippy::large_enum_variant)]
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
    CreateTrigger(CreateTriggerStatement),
    DropTrigger(DropTriggerStatement),
    CreateProcedure(CreateProcedureStatement),
    DropProcedure(DropProcedureStatement),
    Call(CallProcedureStatement),
    Delimiter(DelimiterStatement),
    Analyze(AnalyzeStatement),
    Explain(ExplainStatement),
    Transaction(TransactionStatement),
    Grant(GrantStatement),
    Revoke(RevokeStatement),
    CreateUser(CreateUserStmt),
    DropUser(DropUserStmt),
    ShowStatus,
    ShowProcesslist,
    /// KILL [CONNECTION | QUERY] processlist_id
    Kill(KillStatement),
    /// PREPARE stmt FROM 'sql text'
    Prepare(PrepareStatement),
    /// EXECUTE stmt USING param1, param2, ...
    Execute(ExecuteStatement),
    /// DEALLOCATE PREPARE stmt
    DeallocatePrepare(DeallocatePrepareStatement),
    /// COPY table FROM/TO 'path' (FORMAT PARQUET)
    Copy(CopyStatement),
    /// MERGE INTO target USING source ON condition (SQL-2003)
    Merge(MergeStatement),
    /// TRUNCATE TABLE table_name
    Truncate(TruncateStatement),
}

/// PREPARE statement
#[derive(Debug, Clone, PartialEq)]
pub struct PrepareStatement {
    pub name: String,
    pub sql: String,
}

/// EXECUTE statement
#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteStatement {
    pub name: String,
    pub params: Vec<Expression>,
}

/// DEALLOCATE PREPARE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeallocatePrepareStatement {
    pub name: String,
}

/// KILL statement - KILL [CONNECTION | QUERY] processlist_id
#[derive(Debug, Clone, PartialEq)]
pub struct KillStatement {
    pub process_id: u64,
    pub kill_type: KillType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KillType {
    Connection,
    Query,
}

/// COPY statement - COPY table FROM 'path' (FORMAT PARQUET) or COPY table TO 'path' (FORMAT PARQUET)
#[derive(Debug, Clone, PartialEq)]
pub struct CopyStatement {
    /// Table name
    pub table_name: String,
    /// Direction: true = FROM (import), false = TO (export)
    pub from: bool,
    /// File path
    pub path: String,
    /// Format (only PARQUET supported currently)
    pub format: String,
}

/// MERGE statement - SQL-2003
/// MERGE INTO target USING source ON condition
///   WHEN MATCHED THEN UPDATE SET ...
///   WHEN NOT MATCHED THEN INSERT ...
#[derive(Debug, Clone, PartialEq)]
pub struct MergeStatement {
    pub target_table: String,
    pub source_table: String,
    pub on_condition: Expression,
    pub when_matched: Option<MergeAction>,
    pub when_not_matched: Option<MergeAction>,
}

/// MERGE action (UPDATE or INSERT)
#[derive(Debug, Clone, PartialEq)]
pub enum MergeAction {
    Update {
        set_clauses: Vec<(String, Expression)>,
    },
    Insert {
        columns: Vec<String>,
        values: Vec<Expression>,
    },
}

/// TRUNCATE statement - SQL-2003
/// TRUNCATE TABLE table_name
#[derive(Debug, Clone, PartialEq)]
pub struct TruncateStatement {
    pub table_name: String,
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

/// CREATE TRIGGER statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTriggerStatement {
    pub name: String,
    pub table_name: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub body: Box<Statement>,
}

/// DROP TRIGGER statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropTriggerStatement {
    pub name: String,
}

/// CREATE PROCEDURE statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateProcedureStatement {
    pub name: String,
    pub params: Vec<ProcedureParam>,
    pub body: Vec<ProcedureStatement>,
}

/// Stored procedure parameter
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureParam {
    pub name: String,
    pub mode: ParamMode,
    pub data_type: String,
}

/// Parameter mode for stored procedure
#[derive(Debug, Clone, PartialEq)]
pub enum ParamMode {
    In,
    Out,
    InOut,
}

/// Procedure body statement
#[derive(Debug, Clone, PartialEq)]
pub enum ProcedureStatement {
    /// Raw SQL statement
    RawSql(String),
    /// SELECT ... INTO var1, var2 FROM ...
    SelectInto {
        columns: Vec<String>,
        into_vars: Vec<String>,
        table: String,
        where_clause: Option<String>,
    },
    /// SET variable = value
    Set { variable: String, value: String },
    /// DECLARE variable statement
    Declare {
        name: String,
        data_type: String,
        default_value: Option<String>,
    },
    /// IF condition THEN statements [ELSEIF ...] [ELSE ...] END IF
    If {
        condition: String,
        then_body: Vec<ProcedureStatement>,
        elseif_body: Vec<(String, Vec<ProcedureStatement>)>,
        else_body: Vec<ProcedureStatement>,
    },
    /// WHILE condition DO statements END WHILE
    While {
        condition: String,
        body: Vec<ProcedureStatement>,
    },
    /// LOOP statements END LOOP (with optional LEAVE to exit)
    Loop { body: Vec<ProcedureStatement> },
    /// RETURN expression
    Return { value: String },
    /// LEAVE label - exit a loop
    Leave { label: String },
    /// ITERATE label - continue to next iteration
    Iterate { label: String },
    /// CALL another stored procedure
    Call {
        procedure_name: String,
        args: Vec<String>,
        into_var: Option<String>,
    },
}

/// DROP PROCEDURE statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropProcedureStatement {
    pub name: String,
}

/// CALL statement for invoking stored procedure
#[derive(Debug, Clone, PartialEq)]
pub struct CallProcedureStatement {
    pub procedure_name: String,
    pub args: Vec<String>,
}

/// DELIMITER statement (client-side directive)
#[derive(Debug, Clone, PartialEq)]
pub struct DelimiterStatement {
    pub delimiter: String,
}

/// Trigger timing: BEFORE or AFTER
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerTiming {
    Before,
    After,
}

/// Trigger event: INSERT, UPDATE, or DELETE
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
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
    ReleaseSavepoint { name: String },
}

/// User identity for RBAC: 'username'@'host'
#[derive(Debug, Clone, PartialEq)]
pub struct UserIdentity {
    pub username: String,
    pub host: String,
}

impl UserIdentity {
    pub fn new(username: &str, host: &str) -> Self {
        Self {
            username: username.to_lowercase(),
            host: host.to_lowercase(),
        }
    }
}

/// GRANT statement for permission management
#[derive(Debug, Clone, PartialEq)]
pub struct GrantStatement {
    pub privilege: Privilege,
    pub table: String,
    pub to_user: UserIdentity,
}

/// REVOKE statement for permission management
#[derive(Debug, Clone, PartialEq)]
pub struct RevokeStatement {
    pub privilege: Privilege,
    pub table: String,
    pub from_user: UserIdentity,
}

/// Privilege types
#[derive(Debug, Clone, PartialEq)]
pub enum Privilege {
    Read,
    Write,
    All,
    Process,
    Super,
}

impl Privilege {
    pub fn can_kill(&self) -> bool {
        matches!(self, Privilege::Super | Privilege::All)
    }

    pub fn can_view_processlist(&self) -> bool {
        matches!(self, Privilege::Super | Privilege::Process | Privilege::All)
    }
}

/// CREATE USER statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateUserStmt {
    pub identities: Vec<UserIdentity>,
    pub password: String,
}

/// DROP USER statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropUserStmt {
    pub identities: Vec<UserIdentity>,
}

/// CREATE USER statement
#[derive(Debug, Clone, PartialEq)]
pub struct CreateUserStmt {
    pub identities: Vec<UserIdentity>,
    pub password: String,
}

/// DROP USER statement
#[derive(Debug, Clone, PartialEq)]
pub struct DropUserStmt {
    pub identities: Vec<UserIdentity>,
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

/// GROUP BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByClause {
    pub columns: Vec<Expression>,
}

/// ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

/// ORDER BY single item
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub asc: bool,         // true = ASC, false = DESC
    pub nulls_first: bool, // true = NULLS FIRST, false = NULLS LAST
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
    // New fields for GROUP BY / HAVING / ORDER BY
    pub group_by: Option<GroupByClause>,
    pub having: Option<Expression>,
    pub order_by: Option<OrderByClause>,
    // CTE (Common Table Expression) - SQL-99
    pub with_clause: Option<WithClause>,
}

/// Common Table Expression (CTE) - SQL-99
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub recursive: bool,
    pub cte_tables: Vec<CteTable>,
}

/// CTE table definition
#[derive(Debug, Clone, PartialEq)]
pub struct CteTable {
    pub name: String,
    pub columns: Vec<String>,
    pub query: Box<Statement>,
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
    pub ignore: bool,                 // INSERT IGNORE
    pub replace: bool,                // REPLACE INTO (aliased to INSERT or separate)
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
    pub if_not_exists: bool,
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
    /// Function call expression (for HAVING clause aggregates like COUNT(*), SUM(col))
    FunctionCall(String, Vec<Expression>),
    /// Subquery expression: (SELECT ...)
    Subquery(Box<Statement>),
    /// Qualified column: table.column
    QualifiedColumn(String, String),
    /// Window function expression: ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...)
    WindowFunction {
        func: String,          // Function name: ROW_NUMBER, RANK, LEAD, etc.
        args: Vec<Expression>, // Arguments for LEAD/LAG/NTH_VALUE
        partition_by: Vec<Expression>,
        order_by: Vec<OrderByItem>,
        frame: Option<WindowFrameInfo>,
    },
    /// Parameter placeholder for prepared statements (?)
    Placeholder,
}

/// Window frame info parsed from SQL
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrameInfo {
    pub mode: String, // ROWS, RANGE, or GROUPS
    pub start: FrameBoundInfo,
    pub end: FrameBoundInfo,
    pub exclude: Option<String>, // NO OTHERS, CURRENT ROW, GROUP, or TIES
}

/// Frame bound for window frame
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBoundInfo {
    UnboundedPreceding,
    Preceding(i64),
    CurrentRow,
    Following(i64),
    UnboundedFollowing,
}

/// SQL Parser
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

#[allow(dead_code)]
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
            Some(Token::With) => self.parse_with(),
            Some(Token::Select) => self.parse_select(),
            Some(Token::Insert) | Some(Token::Replace) => self.parse_insert(),
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
            | Some(Token::Savepoint)
            | Some(Token::Release) => self.parse_transaction(),
            Some(Token::Grant) => self.parse_grant(),
            Some(Token::Revoke) => self.parse_revoke(),
            Some(Token::Show) => self.parse_show(),
            Some(Token::Kill) => self.parse_kill(),
            Some(Token::Prepare) => self.parse_prepare(),
            Some(Token::Execute) => self.parse_execute(),
            Some(Token::Deallocate) => self.parse_deallocate(),
            Some(Token::Copy) => self.parse_call(),
            Some(Token::Merge) => self.parse_merge(),
            Some(Token::Truncate) => self.parse_truncate(),
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Empty input".to_string()),
        }
    }

    /// Parse WITH [RECURSIVE] cte AS (SELECT ...) ...
    fn parse_with(&mut self) -> Result<Statement, String> {
        self.expect(Token::With)?;

        let recursive = matches!(self.current(), Some(Token::Recursive));
        if recursive {
            self.next(); // consume RECURSIVE
        }

        let mut cte_tables = Vec::new();

        loop {
            // Parse CTE table name
            let cte_name = match self.current() {
                Some(Token::Identifier(name)) => name.clone(),
                Some(t) => return Err(format!("Expected CTE name, got {:?}", t)),
                None => return Err("Unexpected end of input in CTE".to_string()),
            };
            self.next();

            // Parse column list (optional): WITH RECURSIVE cte(col1, col2) AS (...)
            let mut columns = Vec::new();
            if matches!(self.current(), Some(Token::LParen)) {
                self.next();
                loop {
                    match self.current() {
                        Some(Token::Identifier(name)) => {
                            columns.push(name.clone());
                            self.next();
                            if matches!(self.current(), Some(Token::Comma)) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                        Some(Token::RParen) => {
                            self.next();
                            break;
                        }
                        t => return Err(format!("Expected column name, got {:?}", t)),
                    }
                }
            }

            // Parse AS (SELECT ...)
            self.expect(Token::As)?;
            self.expect(Token::LParen)?;

            // Parse the CTE's SELECT statement
            let cte_select = self.parse_select()?;

            self.expect(Token::RParen)?;

            cte_tables.push(CteTable {
                name: cte_name,
                columns,
                query: Box::new(cte_select),
            });

            // Check for more CTEs or end of WITH clause
            match self.current() {
                Some(Token::Comma) => {
                    self.next(); // consume comma for next CTE
                }
                Some(Token::Select) => {
                    // No more CTEs, main SELECT follows
                    break;
                }
                Some(t) => return Err(format!("Unexpected token after CTE: {:?}", t)),
                None => return Err("Unexpected end of input after CTE".to_string()),
            }
        }

        // Parse the main SELECT statement
        let mut main_select = match self.parse_select()? {
            Statement::Select(s) => s,
            _ => return Err("Expected SELECT after WITH clause".to_string()),
        };

        // Attach WITH clause to the main SELECT
        main_select.with_clause = Some(WithClause {
            recursive,
            cte_tables,
        });

        Ok(Statement::Select(main_select))
    }

    /// Parse MERGE statement - SQL-2003
    /// MERGE INTO target USING source ON condition
    ///   WHEN MATCHED THEN UPDATE SET ...
    ///   WHEN NOT MATCHED THEN INSERT ...
    #[allow(dead_code)]
    fn parse_merge(&mut self) -> Result<Statement, String> {
        self.expect(Token::Merge)?;

        // MERGE INTO target
        self.expect(Token::Into)?;
        let target_table = match self.current() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err("Expected target table name".to_string()),
        };
        self.next();

        // USING source
        self.expect(Token::Using)?;
        let source_table = match self.current() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err("Expected source table name".to_string()),
        };
        self.next();

        // ON condition
        self.expect(Token::On)?;
        let on_condition = self.parse_expression()?;

        let mut when_matched = None;
        let when_not_matched = None;

        // WHEN MATCHED THEN UPDATE SET ... / WHEN NOT MATCHED THEN INSERT ...
        loop {
            match self.current() {
                Some(Token::When) => {
                    self.next();
                    match self.current() {
                        Some(Token::Matched) => {
                            self.next(); // consume MATCHED
                            self.expect(Token::Then)?;
                            self.expect(Token::Update)?;
                            self.expect(Token::Set)?;

                            let mut set_clauses = Vec::new();
                            loop {
                                match self.current() {
                                    Some(Token::Identifier(name)) => {
                                        let col_name = name.clone();
                                        self.next();
                                        self.expect(Token::Equal)?;
                                        let value = self.parse_expression()?;
                                        set_clauses.push((col_name, value));
                                        if matches!(self.current(), Some(Token::Comma)) {
                                            self.next();
                                        } else {
                                            break;
                                        }
                                    }
                                    Some(Token::When) | None => break,
                                    _ => {
                                        return Err("Expected column name in SET clause".to_string())
                                    }
                                }
                            }
                            when_matched = Some(MergeAction::Update { set_clauses });
                        }
                        _ => return Err("Expected MATCHED or NOT MATCHED".to_string()),
                    }
                }
                None => break,
                _ => return Err("Unexpected token in MERGE".to_string()),
            }
        }

        Ok(Statement::Merge(MergeStatement {
            target_table,
            source_table,
            on_condition,
            when_matched,
            when_not_matched,
        }))
    }

    /// Parse TRUNCATE statement - SQL-2003
    /// TRUNCATE TABLE table_name
    fn parse_truncate(&mut self) -> Result<Statement, String> {
        self.expect(Token::Truncate)?;
        self.expect(Token::Table)?;

        let table_name = match self.current() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err("Expected table name".to_string()),
        };
        self.next();

        Ok(Statement::Truncate(TruncateStatement { table_name }))
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.expect(Token::Select)?;

        let mut columns = Vec::new();
        let mut aggregates = Vec::new();
        loop {
            match self.current() {
                Some(Token::From) => break,
                Some(Token::LParen) => {
                    // Subquery in SELECT: (SELECT ...) AS alias
                    self.next();
                    if matches!(self.current(), Some(Token::Select)) {
                        let _select_stmt = self.parse_select()?;
                        self.expect(Token::RParen)?;

                        // Check for AS alias
                        let alias = if matches!(self.current(), Some(Token::As)) {
                            self.next();
                            match self.current() {
                                Some(Token::Identifier(name)) => {
                                    let a = Some(name.clone());
                                    self.next();
                                    a
                                }
                                _ => return Err("Expected alias name".to_string()),
                            }
                        } else {
                            None
                        };

                        columns.push(SelectColumn {
                            name: "(subquery)".to_string(),
                            alias,
                        });
                    } else {
                        return Err("Expected SELECT in subquery".to_string());
                    }
                }
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
                Some(Token::RowNumber)
                | Some(Token::Rank)
                | Some(Token::DenseRank)
                | Some(Token::Lead)
                | Some(Token::Lag)
                | Some(Token::FirstValue)
                | Some(Token::LastValue)
                | Some(Token::NthValue) => {
                    // Parse window function
                    let window_expr = self.parse_window_function()?;
                    columns.push(SelectColumn {
                        name: format!("{:?}", window_expr),
                        alias: None,
                    });
                }
                Some(Token::Identifier(_)) => {
                    let first_name = match self.current() {
                        Some(Token::Identifier(name)) => name.clone(),
                        _ => return Err("Expected column name".to_string()),
                    };
                    self.next();

                    // Check for qualified column: table.column
                    let col_name = if matches!(self.current(), Some(Token::Dot)) {
                        self.next();
                        match self.current() {
                            Some(Token::Identifier(col)) => {
                                let full_name = format!("{}.{}", first_name, col);
                                self.next();
                                full_name
                            }
                            _ => return Err("Expected column name after dot".to_string()),
                        }
                    } else {
                        first_name
                    };

                    // Check for alias: column AS alias or column alias
                    let alias = if matches!(self.current(), Some(Token::As)) {
                        self.next();
                        match self.current() {
                            Some(Token::Identifier(name)) => {
                                let a = Some(name.clone());
                                self.next();
                                a
                            }
                            _ => return Err("Expected alias name".to_string()),
                        }
                    } else {
                        None
                    };

                    columns.push(SelectColumn {
                        name: col_name,
                        alias,
                    });
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
            Some(Token::Identifier(name)) => {
                if matches!(self.current(), Some(Token::Dot)) {
                    self.next();
                    match self.next() {
                        Some(Token::Identifier(table_name)) => {
                            format!("{}.{}", name, table_name)
                        }
                        Some(Token::Processlist) => {
                            format!("{}.processlist", name)
                        }
                        Some(t) => {
                            return Err(format!("Expected table name after dot, got {:?}", t))
                        }
                        None => return Err("Expected table name after dot".to_string()),
                    }
                } else {
                    name
                }
            }
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

        // Parse GROUP BY clause (optional)
        let group_by = if matches!(self.current(), Some(Token::Group)) {
            self.next(); // consume GROUP
            if !matches!(self.current(), Some(Token::By)) {
                return Err("Expected BY after GROUP".to_string());
            }
            self.next(); // consume BY

            let mut columns = Vec::new();
            loop {
                let expr = self.parse_expression()?;
                columns.push(expr);

                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.next(); // consume comma
            }

            Some(GroupByClause { columns })
        } else {
            None
        };

        // Parse HAVING clause (optional) - must follow GROUP BY
        let having = if matches!(self.current(), Some(Token::Having)) {
            self.next(); // consume HAVING
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse ORDER BY clause (optional)
        let order_by = if matches!(self.current(), Some(Token::Order)) {
            self.next(); // consume ORDER
            if !matches!(self.current(), Some(Token::By)) {
                return Err("Expected BY after ORDER".to_string());
            }
            self.next(); // consume BY

            let mut items = Vec::new();
            loop {
                let expr = self.parse_expression()?;

                // Parse ASC/DESC (default ASC)
                let asc = match self.current() {
                    Some(Token::Asc) => {
                        self.next();
                        true
                    }
                    Some(Token::Desc) => {
                        self.next();
                        false
                    }
                    _ => true, // default is ASC
                };

                // Parse NULLS FIRST/LAST (default depends on ASC/DESC)
                let nulls_first = match self.current() {
                    Some(Token::Nulls) => {
                        self.next(); // consume NULLS
                        match self.current() {
                            Some(Token::First) => {
                                self.next();
                                true
                            }
                            Some(Token::Last) => {
                                self.next();
                                false
                            }
                            _ => return Err("Expected FIRST or LAST after NULLS".to_string()),
                        }
                    }
                    _ => asc, // default: NULLS FIRST for ASC, NULLS LAST for DESC
                };

                items.push(OrderByItem {
                    expr,
                    asc,
                    nulls_first,
                });

                if !matches!(self.current(), Some(Token::Comma)) {
                    break;
                }
                self.next(); // consume comma
            }

            Some(OrderByClause { items })
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
            group_by,
            having,
            order_by,
            with_clause: None,
        };

        // Check for LIMIT and OFFSET
        let mut select = base_select.clone();
        if let Some(Token::Limit) = self.current() {
            self.next();
            if let Some(Token::NumberLiteral(n)) = self.current() {
                select.limit = Some(n.parse().unwrap_or(0));
                self.next();
            } else {
                return Err("Expected number after LIMIT".to_string());
            }
        }

        if let Some(Token::Offset) = self.current() {
            self.next();
            if let Some(Token::NumberLiteral(n)) = self.current() {
                select.offset = Some(n.parse().unwrap_or(0));
                self.next();
            } else {
                return Err("Expected number after OFFSET".to_string());
            }
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
        // Check for REPLACE INTO (MySQL syntax: REPLACE INTO table...)
        let mut replace = false;
        let mut ignore = false;

        if let Some(Token::Replace) = self.current() {
            replace = true;
            self.next(); // consume REPLACE
        } else if let Some(Token::Ignore) = self.current() {
            // Check for IGNORE (INSERT IGNORE INTO)
            ignore = true;
            self.next(); // consume IGNORE
        }

        // Now expect INSERT (unless we already consumed REPLACE)
        if !replace {
            self.expect(Token::Insert)?;
        }

        // Check for IGNORE after INSERT (INSERT IGNORE INTO)
        if let Some(Token::Ignore) = self.current() {
            ignore = true;
            self.next(); // consume IGNORE
        }

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

            while let Some(Token::Identifier(name)) = self.current() {
                let col_name = name.clone();
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
                        let v = Expression::Literal(s.to_string());
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
                ignore,
                replace,
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
                        if name.to_uppercase() == "NULL" {
                            row.push(Expression::Literal("NULL".to_string()));
                        } else {
                            row.push(Expression::Identifier(name.clone()));
                        }
                        self.next();
                    }
                    Some(Token::NumberLiteral(n)) => {
                        row.push(Expression::Literal(n.clone()));
                        self.next();
                    }
                    Some(Token::StringLiteral(s)) => {
                        row.push(Expression::Literal(s.to_string()));
                        self.next();
                    }
                    Some(Token::Comma) => {
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
            ignore,
            replace,
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

            // Parse value expression (supports: column = column +/- value)
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

    /// Parse comparison expression (=, !=, >, <, >=, <=)
    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_arithmetic_expression()?;

        // Check for comparison operator
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

        let right = self.parse_arithmetic_expression()?;

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    /// Parse arithmetic expression (+, -, *, /)
    fn parse_arithmetic_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_primary_expression()?;

        // Check for arithmetic operator
        let op = match self.current() {
            Some(Token::Plus) => "+",
            Some(Token::Minus) => "-",
            Some(Token::Asterisk) => "*",
            Some(Token::Slash) => "/",
            _ => return Ok(left), // No operator, return simple expression
        };
        self.next(); // consume operator

        let right = self.parse_arithmetic_expression()?;

        Ok(Expression::BinaryOp(
            Box::new(left),
            op.to_string(),
            Box::new(right),
        ))
    }

    /// Parse primary expression (identifier, literal, or parenthesized)
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        let token = self.current().cloned();

        match token {
            Some(Token::Identifier(name)) => {
                if name.to_uppercase() == "NULL" {
                    return Ok(Expression::Literal("NULL".to_string()));
                }
                self.next();

                // Check for qualified column name: table.column
                if matches!(self.current(), Some(Token::Dot)) {
                    self.next(); // consume '.'
                    match self.current() {
                        Some(Token::Identifier(col_name)) => {
                            let expr = Expression::QualifiedColumn(name.clone(), col_name.clone());
                            self.next();
                            return Ok(expr);
                        }
                        _ => return Err("Expected column name after dot".to_string()),
                    }
                }

                Ok(Expression::Identifier(name.clone()))
            }
            Some(Token::NumberLiteral(n)) => {
                let expr = Expression::Literal(n.clone());
                self.next();
                Ok(expr)
            }
            Some(Token::StringLiteral(s)) => {
                let expr = Expression::Literal(s.to_string());
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
            Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min)
            | Some(Token::Max) => {
                // Parse aggregate function call for HAVING clause
                let func_name = match self.current() {
                    Some(Token::Count) => "COUNT",
                    Some(Token::Sum) => "SUM",
                    Some(Token::Avg) => "AVG",
                    Some(Token::Min) => "MIN",
                    Some(Token::Max) => "MAX",
                    _ => return Err("Unknown aggregate function".to_string()),
                };
                self.next(); // consume function name
                self.expect(Token::LParen)?;

                // Parse arguments
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
                    Some(Token::NumberLiteral(n)) => {
                        args.push(Expression::Literal(n.clone()));
                        self.next();
                    }
                    _ => return Err("Expected *, column name, or number in aggregate".to_string()),
                }

                self.expect(Token::RParen)?;
                Ok(Expression::FunctionCall(func_name.to_string(), args))
            }
            // Window function: ROW_NUMBER() OVER (...)
            Some(Token::RowNumber)
            | Some(Token::Rank)
            | Some(Token::DenseRank)
            | Some(Token::Lead)
            | Some(Token::Lag)
            | Some(Token::FirstValue)
            | Some(Token::LastValue)
            | Some(Token::NthValue) => {
                return self.parse_window_function();
            }
            Some(Token::LParen) => {
                self.next(); // consume '('

                // Check if this is a subquery: (SELECT ...)
                if matches!(self.current(), Some(Token::Select)) {
                    let select_stmt = self.parse_select()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expression::Subquery(Box::new(select_stmt)));
                }

                // Regular parenthesized expression
                let expr = self.parse_or_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::QuestionMark) => {
                self.next(); // consume '?'
                Ok(Expression::Placeholder)
            }
            _ => Err("Expected expression".to_string()),
        }
    }

    /// Parse a window function: ROW_NUMBER() OVER (...)
    fn parse_window_function(&mut self) -> Result<Expression, String> {
        // Parse function name
        let func = match self.current() {
            Some(Token::RowNumber) => "ROW_NUMBER".to_string(),
            Some(Token::Rank) => "RANK".to_string(),
            Some(Token::DenseRank) => "DENSE_RANK".to_string(),
            Some(Token::Lead) => "LEAD".to_string(),
            Some(Token::Lag) => "LAG".to_string(),
            Some(Token::FirstValue) => "FIRST_VALUE".to_string(),
            Some(Token::LastValue) => "LAST_VALUE".to_string(),
            Some(Token::NthValue) => "NTH_VALUE".to_string(),
            Some(Token::Sum) => "SUM".to_string(),
            Some(Token::Avg) => "AVG".to_string(),
            Some(Token::Count) => "COUNT".to_string(),
            Some(Token::Min) => "MIN".to_string(),
            Some(Token::Max) => "MAX".to_string(),
            _ => return Err("Expected window function".to_string()),
        };
        self.next(); // consume function name

        // Parse arguments if any (for LEAD/LAG/NTH_VALUE)
        let mut args = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next(); // consume '('
            if !matches!(self.current(), Some(Token::RParen)) {
                args.push(self.parse_expression()?);
                while matches!(self.current(), Some(Token::Comma)) {
                    self.next(); // consume ','
                    args.push(self.parse_expression()?);
                }
            }
            self.expect(Token::RParen)?;
        }

        // Parse OVER clause
        self.expect(Token::Over)?;
        self.expect(Token::LParen)?;

        // Parse PARTITION BY (optional)
        let mut partition_by = Vec::new();
        if matches!(self.current(), Some(Token::Partition)) {
            self.next(); // consume PARTITION
            self.expect(Token::By)?;
            partition_by.push(self.parse_expression()?);
            while matches!(self.current(), Some(Token::Comma)) {
                self.next(); // consume ','
                partition_by.push(self.parse_expression()?);
            }
        }

        // Parse ORDER BY (optional)
        let mut order_by = Vec::new();
        if matches!(self.current(), Some(Token::Order)) {
            self.next(); // consume ORDER
            self.expect(Token::By)?;
            order_by.push(self.parse_order_by_item()?);
            while matches!(self.current(), Some(Token::Comma)) {
                self.next(); // consume ','
                order_by.push(self.parse_order_by_item()?);
            }
        }

        // Parse window frame (optional)
        let frame = self.parse_window_frame().ok();

        self.expect(Token::RParen)?;

        Ok(Expression::WindowFunction {
            func,
            args,
            partition_by,
            order_by,
            frame,
        })
    }

    /// Parse window frame: ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    fn parse_window_frame(&mut self) -> Result<WindowFrameInfo, String> {
        // Parse frame mode (ROWS, RANGE, or GROUPS) - optional, defaults to ROWS
        let mode = match self.current() {
            Some(Token::Rows) => {
                self.next();
                "ROWS".to_string()
            }
            Some(Token::Range) => {
                self.next();
                "RANGE".to_string()
            }
            Some(Token::Groups) => {
                self.next();
                "GROUPS".to_string()
            }
            // No frame specified - will return early
            _ => {
                return Err("Expected ROWS, RANGE, or GROUPS".to_string());
            }
        };

        // Parse BETWEEN
        self.expect(Token::Between)?;

        let start = self.parse_frame_bound()?;
        self.expect(Token::And)?;
        let end = self.parse_frame_bound()?;

        // Parse EXCLUDE (optional)
        let exclude = if matches!(self.current(), Some(Token::Exclude)) {
            self.next(); // consume EXCLUDE
            Some(match self.current() {
                Some(Token::NoOthers) => {
                    self.next();
                    "NO OTHERS".to_string()
                }
                Some(Token::Current) => {
                    self.next();
                    self.expect(Token::Row)?;
                    "CURRENT ROW".to_string()
                }
                Some(Token::Ties) => {
                    self.next();
                    "TIES".to_string()
                }
                _ => {
                    return Err("Expected NO OTHERS, CURRENT ROW, or TIES after EXCLUDE".to_string())
                }
            })
        } else {
            None
        };

        Ok(WindowFrameInfo {
            mode,
            start,
            end,
            exclude,
        })
    }

    /// Parse frame bound: UNBOUNDED PRECEDING, PRECEDING(n), CURRENT ROW, FOLLOWING(n), UNBOUNDED FOLLOWING
    fn parse_frame_bound(&mut self) -> Result<FrameBoundInfo, String> {
        match self.current() {
            Some(Token::Unbounded) => {
                self.next(); // consume UNBOUNDED
                match self.current() {
                    Some(Token::Preceding) => {
                        self.next();
                        Ok(FrameBoundInfo::UnboundedPreceding)
                    }
                    Some(Token::Following) => {
                        self.next();
                        Ok(FrameBoundInfo::UnboundedFollowing)
                    }
                    _ => Err("Expected PRECEDING or FOLLOWING after UNBOUNDED".to_string()),
                }
            }
            Some(Token::Preceding) => {
                self.next(); // consume PRECEDING
                // Try to parse optional number: PRECEDING(n)
                if matches!(self.current(), Some(Token::LParen)) {
                    self.next(); // consume '('
                    let n = match self.current() {
                        Some(Token::NumberLiteral(n)) => n.parse::<i64>().unwrap_or(1),
                        _ => 1,
                    };
                    if let Some(Token::NumberLiteral(_)) = self.current() {
                        self.next();
                    }
                    self.expect(Token::RParen)?;
                    Ok(FrameBoundInfo::Preceding(n))
                } else {
                    Ok(FrameBoundInfo::Preceding(1))
                }
            }
            Some(Token::Following) => {
                self.next(); // consume FOLLOWING
                // Try to parse optional number: FOLLOWING(n)
                if matches!(self.current(), Some(Token::LParen)) {
                    self.next(); // consume '('
                    let n = match self.current() {
                        Some(Token::NumberLiteral(n)) => n.parse::<i64>().unwrap_or(1),
                        _ => 1,
                    };
                    if let Some(Token::NumberLiteral(_)) = self.current() {
                        self.next();
                    }
                    self.expect(Token::RParen)?;
                    Ok(FrameBoundInfo::Following(n))
                } else {
                    Ok(FrameBoundInfo::Following(1))
                }
            }
            Some(Token::Current) => {
                self.next(); // consume CURRENT
                self.expect(Token::Row)?;
                Ok(FrameBoundInfo::CurrentRow)
            }
            _ => Err("Expected frame bound (UNBOUNDED PRECEDING, PRECEDING, CURRENT ROW, FOLLOWING, UNBOUNDED FOLLOWING)".to_string()),
        }
    }

    /// Parse ORDER BY item
    fn parse_order_by_item(&mut self) -> Result<OrderByItem, String> {
        let expr = self.parse_expression()?;

        // Parse ASC/DESC (default ASC)
        let asc = match self.current() {
            Some(Token::Asc) => {
                self.next();
                true
            }
            Some(Token::Desc) => {
                self.next();
                false
            }
            _ => true, // default is ASC
        };

        // Parse NULLS FIRST/LAST (default depends on ASC/DESC)
        let nulls_first = match self.current() {
            Some(Token::Nulls) => {
                self.next(); // consume NULLS
                match self.current() {
                    Some(Token::First) => {
                        self.next();
                        true
                    }
                    Some(Token::Last) => {
                        self.next();
                        false
                    }
                    _ => return Err("Expected FIRST or LAST after NULLS".to_string()),
                }
            }
            _ => asc, // default: NULLS FIRST for ASC, NULLS LAST for DESC
        };

        Ok(OrderByItem {
            expr,
            asc,
            nulls_first,
        })
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

    #[allow(dead_code)]
    fn parse_drop_index(&mut self) -> Result<Statement, String> {
        // DROP INDEX index_name
        self.expect(Token::Index)?;
        let name = match self.next() {
            Some(Token::Identifier(n)) => n,
            _ => return Err("Expected index name".to_string()),
        };

        Ok(Statement::DropIndex(DropIndexStatement { name }))
    }

    #[allow(dead_code)]
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    fn parse_create_or_index(&mut self) -> Result<Statement, String> {
        let mut pos = self.position;
        let mut is_index = false;
        let mut is_user = false;
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
        if pos < self.tokens.len() {
            if let Some(Token::User) = &self.tokens.get(pos) {
                is_user = true;
            }
        }
        if is_index {
            self.next();
            self.parse_create_index()
        } else if is_user {
            self.next();
            self.parse_create_user()
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
            Some(Token::Trigger) => {
                self.next();
                self.parse_create_trigger()
            }
            Some(Token::Procedure) => {
                self.next();
                self.parse_create_procedure()
            }
            _ => Err("Expected TABLE, VIEW, TRIGGER, or PROCEDURE after CREATE".to_string()),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        // Parse optional IF NOT EXISTS
        let mut if_not_exists = false;
        if matches!(self.current(), Some(Token::If)) {
            self.next(); // consume IF
            if matches!(self.current(), Some(Token::Not)) {
                self.next(); // consume NOT
                if let Some(Token::Identifier(s)) = self.current() {
                    if s.to_uppercase() == "EXISTS" {
                        if_not_exists = true;
                        self.next(); // consume EXISTS
                    }
                }
            }
        }

        let name = match self.current() {
            Some(Token::Identifier(name)) => {
                let n = name.clone();
                self.next();
                n
            }
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
                                    // Handle optional parentheses: REFERENCES table(column)
                                    let ref_column = if let Some(Token::LParen) = self.current() {
                                        // Consume '('
                                        self.next();
                                        let col = match self.current() {
                                            Some(Token::Identifier(s)) => {
                                                let c = s.clone();
                                                self.next();
                                                c
                                            }
                                            _ => "id".to_string(),
                                        };
                                        // Consume ')' if present
                                        if let Some(Token::RParen) = self.current() {
                                            self.next();
                                        }
                                        col
                                    } else {
                                        match self.current() {
                                            Some(Token::Identifier(s)) => {
                                                let c = s.clone();
                                                self.next();
                                                c
                                            }
                                            _ => "id".to_string(),
                                        }
                                    };
                                    references = Some(ForeignKeyRef {
                                        table: ref_table,
                                        column: ref_column,
                                        on_delete: None,
                                        on_update: None,
                                    });
                                    // Parse ON DELETE and ON UPDATE actions
                                    while let Some(Token::On) = self.current() {
                                        self.next();
                                        // Parse ON DELETE action
                                        if let Some(Token::Delete) = self.current() {
                                            self.next();
                                            // Check for SET NULL or direct action
                                            let action = match self.current() {
                                                Some(Token::Set) => {
                                                    self.next();
                                                    if let Some(Token::Identifier(name)) =
                                                        self.current()
                                                    {
                                                        if name.to_uppercase() == "NULL" {
                                                            self.next();
                                                            Some(ForeignKeyAction::SetNull)
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    }
                                                }
                                                Some(Token::Identifier(action)) => {
                                                    let action_upper = action.to_uppercase();
                                                    self.next();
                                                    match action_upper.as_str() {
                                                        "CASCADE" => {
                                                            Some(ForeignKeyAction::Cascade)
                                                        }
                                                        "RESTRICT" => {
                                                            Some(ForeignKeyAction::Restrict)
                                                        }
                                                        _ => None,
                                                    }
                                                }
                                                _ => None,
                                            };
                                            if let Some(ref mut fk_ref) = references {
                                                fk_ref.on_delete = action;
                                            }
                                        }
                                        // Check if we're at UPDATE before checking for ON (order matters!)
                                        if let Some(Token::Update) = self.current() {
                                            self.next();
                                            let action = match self.current() {
                                                Some(Token::Set) => {
                                                    self.next();
                                                    if let Some(Token::Identifier(name)) =
                                                        self.current()
                                                    {
                                                        if name.to_uppercase() == "NULL" {
                                                            self.next();
                                                            Some(ForeignKeyAction::SetNull)
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    }
                                                }
                                                Some(Token::Identifier(action)) => {
                                                    let action_upper = action.to_uppercase();
                                                    self.next();
                                                    match action_upper.as_str() {
                                                        "CASCADE" => {
                                                            Some(ForeignKeyAction::Cascade)
                                                        }
                                                        "RESTRICT" => {
                                                            Some(ForeignKeyAction::Restrict)
                                                        }
                                                        _ => None,
                                                    }
                                                }
                                                _ => None,
                                            };
                                            if let Some(ref mut fk_ref) = references {
                                                fk_ref.on_update = action;
                                            }
                                        }
                                    }
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
            if_not_exists,
        }))
    }

    fn parse_drop_table(&mut self) -> Result<Statement, String> {
        let mut pos = self.position;
        let mut is_user = false;
        if pos < self.tokens.len() {
            pos += 1;
        }
        if pos < self.tokens.len() {
            if let Some(Token::User) = &self.tokens.get(pos) {
                is_user = true;
            }
        }
        if is_user {
            self.next();
            self.parse_drop_user()
        } else {
            self.expect(Token::Drop)?;
            self.expect(Token::Table)?;
            let name = match self.next() {
                Some(Token::Identifier(name)) => name,
                _ => return Err("Expected table name".to_string()),
            };
            Ok(Statement::DropTable(DropTableStatement { name }))
        }
    }

    fn parse_drop_trigger(&mut self) -> Result<Statement, String> {
        self.expect(Token::Trigger)?;
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected trigger name".to_string()),
        };

        Ok(Statement::DropTrigger(DropTriggerStatement { name }))
    }

    fn parse_create_user(&mut self) -> Result<Statement, String> {
        self.expect(Token::User)?;

        let mut identities = Vec::new();
        loop {
            let username = match self.current() {
                Some(Token::StringLiteral(s)) => s.clone(),
                _ => return Err("Expected username string".to_string()),
            };
            self.next();

            let host = if matches!(self.current(), Some(Token::At)) {
                self.next();
                let host_str = match self.current() {
                    Some(Token::StringLiteral(s)) => s.clone(),
                    _ => return Err("Expected host string after @".to_string()),
                };
                self.next();
                host_str
            } else {
                "%".to_string()
            };

            identities.push(UserIdentity::new(&username, &host));

            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        self.expect(Token::Identified)?;
        self.expect(Token::By)?;
        match self.current() {
            Some(Token::StringLiteral(pwd)) => {
                let password = pwd.clone();
                self.next();
                Ok(Statement::CreateUser(CreateUserStmt {
                    identities,
                    password,
                }))
            }
            _ => Err("Expected password after IDENTIFIED BY".to_string()),
        }
    }

    fn parse_drop_user(&mut self) -> Result<Statement, String> {
        self.expect(Token::User)?;

        let mut identities = Vec::new();
        loop {
            let username = match self.current() {
                Some(Token::StringLiteral(s)) => s.clone(),
                _ => return Err("Expected username string".to_string()),
            };
            self.next();

            let host = if matches!(self.current(), Some(Token::At)) {
                self.next();
                let host_str = match self.current() {
                    Some(Token::StringLiteral(s)) => s.clone(),
                    _ => return Err("Expected host string after @".to_string()),
                };
                self.next();
                host_str
            } else {
                "%".to_string()
            };

            identities.push(UserIdentity::new(&username, &host));

            if !matches!(self.current(), Some(Token::Comma)) {
                break;
            }
            self.next();
        }

        Ok(Statement::DropUser(DropUserStmt { identities }))
    }

    fn parse_create_trigger(&mut self) -> Result<Statement, String> {
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected trigger name".to_string()),
        };

        // Parse ON keyword
        self.expect(Token::On)?;

        let table_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected table name".to_string()),
        };

        // Parse timing: BEFORE or AFTER
        let timing = match self.current() {
            Some(Token::Before) => {
                self.next();
                TriggerTiming::Before
            }
            Some(Token::After) => {
                self.next();
                TriggerTiming::After
            }
            _ => return Err("Expected BEFORE or AFTER".to_string()),
        };

        // Parse event: INSERT, UPDATE, DELETE
        let event = match self.current() {
            Some(Token::Insert) => {
                self.next();
                TriggerEvent::Insert
            }
            Some(Token::Update) => {
                self.next();
                TriggerEvent::Update
            }
            Some(Token::Delete) => {
                self.next();
                TriggerEvent::Delete
            }
            _ => return Err("Expected INSERT, UPDATE, or DELETE".to_string()),
        };

        // Parse DO keyword
        self.expect(Token::Do)?;

        // Parse trigger body (simple: single statement)
        let body = match self.current() {
            Some(Token::Insert) => Box::new(self.parse_insert()?),
            Some(Token::Update) => Box::new(self.parse_update()?),
            Some(Token::Delete) => Box::new(self.parse_delete()?),
            Some(Token::Select) => Box::new(self.parse_select()?),
            _ => {
                return Err("Expected INSERT, UPDATE, DELETE, or SELECT in trigger body".to_string())
            }
        };

        Ok(Statement::CreateTrigger(CreateTriggerStatement {
            name,
            table_name,
            timing,
            event,
            body,
        }))
    }

    fn parse_create_procedure(&mut self) -> Result<Statement, String> {
        // Parse procedure name
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected procedure name".to_string()),
        };

        // Parse parameters: (param1, param2, ...)
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.current(), Some(Token::RParen)) {
            loop {
                let param_name = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    _ => return Err("Expected parameter name".to_string()),
                };

                // Parse parameter mode (IN, OUT, INOUT) - default is IN
                let mode = match self.current() {
                    Some(Token::Identifier(mode_str)) => {
                        match mode_str.to_uppercase().as_str() {
                            "IN" => {
                                self.next();
                                ParamMode::In
                            }
                            "OUT" => {
                                self.next();
                                ParamMode::Out
                            }
                            "INOUT" => {
                                self.next();
                                ParamMode::InOut
                            }
                            _ => ParamMode::In, // default
                        }
                    }
                    _ => ParamMode::In,
                };

                // Parse data type
                let data_type = match self.next() {
                    Some(Token::Identifier(dt)) => dt,
                    Some(Token::Integer) => "INTEGER".to_string(),
                    Some(Token::Text) => "TEXT".to_string(),
                    Some(Token::Float) => "FLOAT".to_string(),
                    Some(Token::Decimal) => "DECIMAL".to_string(),
                    Some(Token::Boolean) => "BOOLEAN".to_string(),
                    Some(Token::Date) => "DATE".to_string(),
                    Some(Token::Timestamp) => "TIMESTAMP".to_string(),
                    Some(Token::Blob) => "BLOB".to_string(),
                    _ => return Err("Expected data type".to_string()),
                };

                params.push(ProcedureParam {
                    name: param_name,
                    mode,
                    data_type,
                });

                match self.current() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }
        self.expect(Token::RParen)?;

        // Parse BEGIN...END block
        self.expect(Token::Begin)?;

        let mut body = Vec::new();

        // Simple body parsing: collect statements until END
        // Note: This is a simplified implementation that stores raw SQL for now
        while !matches!(self.current(), Some(Token::Identifier(end_str)) 
                       if end_str.to_uppercase() == "END")
            && !matches!(self.current(), None)
        {
            let stmt = match self.current() {
                Some(Token::Select) => {
                    // For now, just collect SELECT statements as raw SQL
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Set) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Identifier(_)) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Semicolon) => {
                    self.next();
                    continue;
                }
                Some(Token::If) => {
                    let raw_sql = self.collect_until_end_if();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::While) => {
                    let raw_sql = self.collect_until_end_loop();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Loop) => {
                    let raw_sql = self.collect_until_end_loop();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Leave) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Iterate) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Signal) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                Some(Token::Return) => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
                _ => {
                    let raw_sql = self.collect_until_semicolon();
                    ProcedureStatement::RawSql(raw_sql)
                }
            };
            body.push(stmt);
        }

        // Expect END
        if matches!(self.current(), Some(Token::Identifier(end_str)) 
                   if end_str.to_uppercase() == "END")
        {
            self.next();
        }

        Ok(Statement::CreateProcedure(CreateProcedureStatement {
            name,
            params,
            body,
        }))
    }

    /// Parse procedure body statements until END (but don't consume END)
    fn parse_procedure_body(&mut self) -> Result<Vec<ProcedureStatement>, String> {
        let mut body = Vec::new();

        loop {
            // Check for END before parsing statement
            if matches!(self.current(), Some(Token::Identifier(end_str)) 
                       if end_str.to_uppercase() == "END")
            {
                break;
            }

            if self.current().is_none() {
                break;
            }

            match self.parse_procedure_statement() {
                Ok(stmt) => {
                    // Skip empty statements
                    if !matches!(stmt, ProcedureStatement::RawSql(ref s) if s.is_empty()) {
                        body.push(stmt);
                    }
                }
                Err(e) if e == "END" || e == "ELSE" || e == "ELSEIF" => break, // Control flow signals
                Err(e) => return Err(e),
            }
        }

        // Don't consume END - let the caller handle it
        Ok(body)
    }

    /// Parse a single procedure statement
    fn parse_procedure_statement(&mut self) -> Result<ProcedureStatement, String> {
        match self.current() {
            None => Err("Unexpected end of input".to_string()),
            Some(Token::Semicolon) => {
                self.next();
                Ok(ProcedureStatement::RawSql(String::new())) // Skip empty statements
            }
            // END/ELSE/ELSEIF signals end of procedure body - return error to signal caller to stop
            Some(Token::Identifier(id)) if id.to_uppercase() == "END" => {
                Err("END".to_string()) // Signal to caller to stop
            }
            Some(Token::Else) => Err("ELSE".to_string()), // Signal ELSE to caller
            Some(Token::Elsif) => Err("ELSEIF".to_string()), // Signal ELSEIF to caller
            Some(Token::Declare) => self.parse_procedure_declare(),
            Some(Token::If) => self.parse_procedure_if(),
            Some(Token::While) => self.parse_procedure_while(),
            Some(Token::Loop) => self.parse_procedure_loop(),
            Some(Token::Leave) => self.parse_procedure_leave(),
            Some(Token::Iterate) => self.parse_procedure_iterate(),
            Some(Token::Return) => self.parse_procedure_return(),
            Some(Token::Call) => self.parse_procedure_call(),
            Some(Token::Set) => self.parse_procedure_set(),
            Some(Token::Select) => {
                let sql = self.collect_until_semicolon();
                Ok(ProcedureStatement::RawSql(sql))
            }
            Some(Token::Identifier(id)) if id.to_uppercase() == "CALL" => {
                self.next(); // consume CALL
                self.parse_procedure_call()
            }
            _ => {
                let sql = self.collect_until_semicolon();
                Ok(ProcedureStatement::RawSql(sql))
            }
        }
    }

    /// Parse DECLARE variable statement
    fn parse_procedure_declare(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Declare)?;

        // Variable name
        let name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected variable name after DECLARE".to_string()),
        };

        // Data type
        let data_type = match self.current() {
            Some(Token::Identifier(dt)) => {
                let dt = dt.clone();
                self.next();
                dt
            }
            Some(Token::Integer) => {
                self.next();
                "INTEGER".to_string()
            }
            Some(Token::Text) => {
                self.next();
                "TEXT".to_string()
            }
            _ => return Err("Expected data type after DECLARE".to_string()),
        };

        // Optional DEFAULT value
        let default_value = if matches!(self.current(), Some(Token::Identifier(id)) if id.to_uppercase() == "DEFAULT")
        {
            self.next();
            let value = self.collect_until_semicolon().trim().to_string();
            Some(value)
        } else {
            self.expect(Token::Semicolon)?;
            None
        };

        Ok(ProcedureStatement::Declare {
            name,
            data_type,
            default_value,
        })
    }

    /// Parse IF condition THEN ... END IF statement
    fn parse_procedure_if(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::If)?;

        // Condition
        let condition = self.collect_until_token(&[Token::Then]);

        self.expect(Token::Then)?;

        // THEN body
        let then_body = self.parse_procedure_body()?;

        // ELSEIF branches
        let mut elseif_body = Vec::new();
        while matches!(self.current(), Some(Token::Elsif)) {
            self.next(); // consume ELSEIF
            let elseif_condition = self.collect_until_token(&[Token::Then]);
            self.expect(Token::Then)?;
            let elseif_then_body = self.parse_procedure_body()?;
            elseif_body.push((elseif_condition, elseif_then_body));
        }

        // ELSE body
        let else_body = if matches!(self.current(), Some(Token::Else)) {
            self.next(); // consume ELSE
            self.parse_procedure_body()?
        } else {
            Vec::new()
        };

        self.expect_token_case_insensitive("END")?;
        self.expect_token_case_insensitive("IF")?;

        Ok(ProcedureStatement::If {
            condition,
            then_body,
            elseif_body,
            else_body,
        })
    }

    /// Parse WHILE condition DO ... END WHILE statement
    fn parse_procedure_while(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::While)?;

        let condition = self.collect_until_token(&[Token::Do]);
        self.expect(Token::Do)?;

        let body = self.parse_procedure_body()?;

        self.expect_token_case_insensitive("END")?;
        self.expect_token_case_insensitive("WHILE")?;

        Ok(ProcedureStatement::While { condition, body })
    }

    /// Parse LOOP ... END LOOP statement
    fn parse_procedure_loop(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Loop)?;

        let body = self.parse_procedure_body()?;

        self.expect_token_case_insensitive("END")?;
        self.expect_token_case_insensitive("LOOP")?;

        Ok(ProcedureStatement::Loop { body })
    }

    /// Parse LEAVE label statement
    fn parse_procedure_leave(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Leave)?;
        let label = match self.next() {
            Some(Token::Identifier(id)) => id,
            _ => return Err("Expected label after LEAVE".to_string()),
        };
        self.expect(Token::Semicolon)?;
        Ok(ProcedureStatement::Leave { label })
    }

    /// Parse ITERATE label statement
    fn parse_procedure_iterate(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Iterate)?;
        let label = match self.next() {
            Some(Token::Identifier(id)) => id,
            _ => return Err("Expected label after ITERATE".to_string()),
        };
        self.expect(Token::Semicolon)?;
        Ok(ProcedureStatement::Iterate { label })
    }

    /// Parse RETURN expression statement
    fn parse_procedure_return(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Return)?;
        let value = self.collect_until_semicolon();
        Ok(ProcedureStatement::Return { value })
    }

    /// Parse CALL statement for stored procedure invocation
    fn parse_procedure_call(&mut self) -> Result<ProcedureStatement, String> {
        // Procedure name
        let procedure_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected procedure name".to_string()),
        };

        // Arguments
        let mut args = Vec::new();
        if matches!(self.current(), Some(Token::LParen)) {
            self.next(); // consume (
            while !matches!(self.current(), Some(Token::RParen) | None) {
                match self.current() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    Some(Token::Identifier(id)) => {
                        args.push(id.clone());
                        self.next();
                    }
                    _ => break,
                }
            }
            self.expect(Token::RParen)?;
        }

        // Optional INTO variable
        let into_var = if matches!(self.current(), Some(Token::Identifier(id)) if id.to_uppercase() == "INTO")
        {
            self.next();
            match self.next() {
                Some(Token::Identifier(name)) => Some(name),
                _ => return Err("Expected variable name after INTO".to_string()),
            }
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Ok(ProcedureStatement::Call {
            procedure_name,
            args,
            into_var,
        })
    }

    /// Parse SET variable = value statement
    fn parse_procedure_set(&mut self) -> Result<ProcedureStatement, String> {
        self.expect(Token::Set)?;
        let variable = match self.next() {
            Some(Token::Identifier(id)) => id,
            _ => return Err("Expected variable name".to_string()),
        };

        // Handle = or := assignment
        if matches!(self.current(), Some(Token::Equal)) {
            self.next();
        }

        let value = self.collect_until_semicolon();

        Ok(ProcedureStatement::Set { variable, value })
    }

    /// Collect tokens until one of the specified tokens is encountered
    fn collect_until_token(&mut self, tokens: &[Token]) -> String {
        let mut result = String::new();
        let mut paren_depth = 0;

        loop {
            match self.current() {
                None => break,
                Some(t) if tokens.contains(&t) && paren_depth == 0 => break,
                Some(Token::LParen) => {
                    paren_depth += 1;
                    result.push('(');
                    self.next();
                }
                Some(Token::RParen) if paren_depth > 0 => {
                    paren_depth -= 1;
                    result.push(')');
                    self.next();
                }
                Some(tok) => {
                    if !result.is_empty() && !result.ends_with('(') {
                        result.push(' ');
                    }
                    result.push_str(&tok.to_string());
                    self.next();
                }
            }
        }
        result.trim().to_string()
    }

    /// Expect a token case-insensitively for identifiers or keywords
    fn expect_token_case_insensitive(&mut self, expected: &str) -> Result<(), String> {
        match self.current() {
            // Handle Token::Identifier
            Some(Token::Identifier(id)) if id.to_uppercase() == expected.to_uppercase() => {
                self.next();
                Ok(())
            }
            // Handle keyword tokens that match the expected string
            Some(Token::While) if "WHILE".eq_ignore_ascii_case(expected) => {
                self.next();
                Ok(())
            }
            Some(Token::Loop) if "LOOP".eq_ignore_ascii_case(expected) => {
                self.next();
                Ok(())
            }
            Some(Token::If) | Some(Token::EndIf) if "IF".eq_ignore_ascii_case(expected) => {
                self.next();
                Ok(())
            }
            Some(Token::Else) if "ELSE".eq_ignore_ascii_case(expected) => {
                self.next();
                Ok(())
            }
            _ => Err(format!("Expected '{}'", expected)),
        }
    }

    /// Collect tokens until semicolon (exclusive)
    fn collect_until_semicolon(&mut self) -> String {
        let mut sql = String::new();
        loop {
            match self.current() {
                Some(Token::Semicolon) => {
                    self.next();
                    break;
                }
                Some(Token::Eof) => break,
                None => break,
                Some(tok) => {
                    sql.push_str(&tok.to_string());
                    sql.push(' ');
                    self.next();
                }
            }
        }
        sql.trim().to_string()
    }

    /// Collect tokens until END IF
    fn collect_until_end_if(&mut self) -> String {
        let mut sql = String::new();
        let mut end_if_depth = 0;
        loop {
            match self.current() {
                Some(Token::Eof) | None => break,
                Some(Token::Identifier(id)) if id.to_uppercase() == "END" => {
                    sql.push_str("END");
                    sql.push(' ');
                    self.next();
                    if matches!(self.current(), Some(Token::If)) {
                        if end_if_depth > 0 {
                            end_if_depth -= 1;
                            sql.push_str("IF ");
                            self.next();
                        } else {
                            sql.push_str("IF");
                            self.next();
                            break;
                        }
                    }
                }
                Some(Token::If) => {
                    end_if_depth += 1;
                    sql.push_str("IF ");
                    self.next();
                }
                Some(tok) => {
                    sql.push_str(&tok.to_string());
                    sql.push(' ');
                    self.next();
                }
            }
        }
        sql.trim().to_string()
    }

    /// Collect tokens until END LOOP
    fn collect_until_end_loop(&mut self) -> String {
        let mut sql = String::new();
        let mut end_loop_depth = 0;
        loop {
            match self.current() {
                Some(Token::Eof) | None => break,
                Some(Token::Identifier(id)) if id.to_uppercase() == "END" => {
                    sql.push_str("END");
                    sql.push(' ');
                    self.next();
                    if matches!(self.current(), Some(Token::Loop)) {
                        if end_loop_depth > 0 {
                            end_loop_depth -= 1;
                            sql.push_str("LOOP ");
                            self.next();
                        } else {
                            sql.push_str("LOOP");
                            self.next();
                            break;
                        }
                    }
                }
                Some(Token::Loop) => {
                    end_loop_depth += 1;
                    sql.push_str("LOOP ");
                    self.next();
                }
                Some(tok) => {
                    sql.push_str(&tok.to_string());
                    sql.push(' ');
                    self.next();
                }
            }
        }
        sql.trim().to_string()
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
                UserIdentity::new(&u, "%")
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
                UserIdentity::new(&u, "%")
            }
            _ => return Err("Expected user name".to_string()),
        };

        Ok(Statement::Revoke(RevokeStatement {
            privilege,
            table,
            from_user,
        }))
    }

    fn parse_show(&mut self) -> Result<Statement, String> {
        self.expect(Token::Show)?;

        match self.current() {
            Some(Token::Status) => {
                self.next();
                Ok(Statement::ShowStatus)
            }
            Some(Token::Processlist) => {
                self.next();
                Ok(Statement::ShowProcesslist)
            }
            _ => Err("Expected STATUS or PROCESSLIST after SHOW".to_string()),
        }
    }

    /// Parse KILL statement: KILL [CONNECTION | QUERY] process_id
    fn parse_kill(&mut self) -> Result<Statement, String> {
        self.expect(Token::Kill)?;

        let kill_type = match self.current() {
            Some(Token::Connection) => {
                self.next();
                KillType::Connection
            }
            Some(Token::Query) => {
                self.next();
                KillType::Query
            }
            _ => KillType::Connection,
        };

        let process_id = match self.current() {
            Some(Token::NumberLiteral(id)) => {
                let id_str = id.clone();
                self.next();
                id_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid process ID: {}", id_str))?
            }
            _ => return Err("Expected process ID after KILL".to_string()),
        };

        Ok(Statement::Kill(KillStatement {
            process_id,
            kill_type,
        }))
    }

    /// Parse PREPARE statement: PREPARE stmt FROM 'sql text'
    fn parse_prepare(&mut self) -> Result<Statement, String> {
        self.expect(Token::Prepare)?;

        // Get statement name
        let name = match self.current() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.next();
                name
            }
            _ => return Err("Expected statement name after PREPARE".to_string()),
        };

        self.expect(Token::From)?;

        // Get SQL string literal
        let sql = match self.current() {
            Some(Token::StringLiteral(s)) => {
                let sql = s.clone();
                self.next();
                sql
            }
            _ => return Err("Expected SQL string literal after FROM".to_string()),
        };

        Ok(Statement::Prepare(PrepareStatement { name, sql }))
    }

    /// Parse EXECUTE statement: EXECUTE stmt USING param1, param2, ...
    fn parse_execute(&mut self) -> Result<Statement, String> {
        self.expect(Token::Execute)?;

        // Get statement name
        let name = match self.current() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.next();
                name
            }
            _ => return Err("Expected statement name after EXECUTE".to_string()),
        };

        // Parse optional USING clause
        let mut params = Vec::new();
        if let Some(Token::Using) = self.current() {
            self.next();
            // Parse parameter list
            loop {
                params.push(self.parse_expression()?);
                match self.current() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }

        Ok(Statement::Execute(ExecuteStatement { name, params }))
    }

    /// Parse DEALLOCATE PREPARE statement: DEALLOCATE PREPARE stmt
    fn parse_deallocate(&mut self) -> Result<Statement, String> {
        self.expect(Token::Deallocate)?;
        self.expect(Token::Prepare)?;

        // Get statement name
        let name = match self.current() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.next();
                name
            }
            _ => return Err("Expected statement name after DEALLOCATE PREPARE".to_string()),
        };

        Ok(Statement::DeallocatePrepare(DeallocatePrepareStatement {
            name,
        }))
    }

    fn parse_call(&mut self) -> Result<Statement, String> {
        self.expect(Token::Call)?;

        // Get procedure name
        let procedure_name = match self.next() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected procedure name".to_string()),
        };

        // Parse arguments: (arg1, arg2, ...)
        self.expect(Token::LParen)?;

        let mut args = Vec::new();
        if !matches!(self.current(), Some(Token::RParen)) {
            loop {
                let arg = match self.next() {
                    Some(Token::Identifier(name)) => name,
                    Some(Token::StringLiteral(s)) => s,
                    Some(Token::NumberLiteral(n)) => n,
                    Some(Token::QuestionMark) => "?".to_string(),
                    _ => return Err("Expected argument".to_string()),
                };
                args.push(arg);

                match self.current() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    _ => break,
                }
            }
        }
        self.expect(Token::RParen)?;

        Ok(Statement::Call(CallProcedureStatement {
            procedure_name,
            args,
        }))
    }

    fn parse_delimiter(&mut self) -> Result<Statement, String> {
        self.expect(Token::Delimiter)?;

        // Get the new delimiter
        let delimiter = match self.next() {
            Some(Token::Semicolon) => ";".to_string(),
            Some(Token::Identifier(d)) => d,
            Some(Token::Eof) | None => ";".to_string(),
            _ => return Err("Expected delimiter".to_string()),
        };

        Ok(Statement::Delimiter(DelimiterStatement { delimiter }))
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
            Some(Token::Release) => {
                self.next();
                // Expect SAVEPOINT after RELEASE
                if !matches!(self.current(), Some(Token::Savepoint)) {
                    return Err("Expected SAVEPOINT after RELEASE".to_string());
                }
                self.next(); // consume SAVEPOINT
                let name = match self.current() {
                    Some(Token::Identifier(n)) => {
                        let name = n.clone();
                        self.next();
                        name
                    }
                    _ => return Err("Expected savepoint name".to_string()),
                };
                Ok(Statement::Transaction(TransactionStatement {
                    command: TransactionCommand::ReleaseSavepoint { name },
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
                assert_eq!(g.to_user, UserIdentity::new("alice", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("bob", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("admin", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("guest", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("writer", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("updater", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("deleter", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("alice", "%"));
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
                assert_eq!(g.to_user, UserIdentity::new("alice", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("alice", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("bob", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("admin", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("guest", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("writer", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("updater", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("deleter", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("alice", "%"));
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
                assert_eq!(r.from_user, UserIdentity::new("alice", "%"));
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
            to_user: UserIdentity::new("alice", "%"),
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
            from_user: UserIdentity::new("bob", "%"),
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
            to_user: UserIdentity::new("alice", "%"),
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
            from_user: UserIdentity::new("bob", "%"),
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
    fn test_parse_create_user() {
        let result = parse("CREATE USER 'alice'@'localhost' IDENTIFIED BY 'password123'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateUser(c) => {
                assert_eq!(c.identities.len(), 1);
                assert_eq!(c.identities[0].username, "alice");
                assert_eq!(c.identities[0].host, "localhost");
                assert_eq!(c.password, "password123");
            }
            _ => panic!("Expected CREATE USER statement"),
        }
    }

    #[test]
    fn test_parse_create_user_multiple() {
        let result = parse("CREATE USER 'alice'@'localhost', 'bob'@'%' IDENTIFIED BY 'password'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateUser(c) => {
                assert_eq!(c.identities.len(), 2);
                assert_eq!(c.identities[0].username, "alice");
                assert_eq!(c.identities[0].host, "localhost");
                assert_eq!(c.identities[1].username, "bob");
                assert_eq!(c.identities[1].host, "%");
                assert_eq!(c.password, "password");
            }
            _ => panic!("Expected CREATE USER statement"),
        }
    }

    #[test]
    fn test_parse_create_user_default_host() {
        let result = parse("CREATE USER 'alice' IDENTIFIED BY 'password'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateUser(c) => {
                assert_eq!(c.identities.len(), 1);
                assert_eq!(c.identities[0].username, "alice");
                assert_eq!(c.identities[0].host, "%");
            }
            _ => panic!("Expected CREATE USER statement"),
        }
    }

    #[test]
    fn test_parse_drop_user() {
        let result = parse("DROP USER 'alice'@'localhost'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::DropUser(d) => {
                assert_eq!(d.identities.len(), 1);
                assert_eq!(d.identities[0].username, "alice");
                assert_eq!(d.identities[0].host, "localhost");
            }
            _ => panic!("Expected DROP USER statement"),
        }
    }

    #[test]
    fn test_parse_drop_user_multiple() {
        let result = parse("DROP USER 'alice'@'localhost', 'bob'@'%'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::DropUser(d) => {
                assert_eq!(d.identities.len(), 2);
                assert_eq!(d.identities[0].username, "alice");
                assert_eq!(d.identities[0].host, "localhost");
                assert_eq!(d.identities[1].username, "bob");
                assert_eq!(d.identities[1].host, "%");
            }
            _ => panic!("Expected DROP USER statement"),
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
            ignore: false,
            replace: false,
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
    fn test_parse_multiple_foreign_keys() {
        // Test that multiple FK columns can be parsed correctly
        let result = parse(
            "CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id), product_id INTEGER REFERENCES products(id))",
        );
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::CreateTable(ct) => {
                assert_eq!(ct.columns.len(), 3, "Should parse all 3 columns");
                // First column: id
                assert_eq!(ct.columns[0].name, "id");
                assert!(ct.columns[0].references.is_none());
                // Second column: user_id with FK to users
                assert_eq!(ct.columns[1].name, "user_id");
                assert!(ct.columns[1].references.is_some());
                let fk1 = ct.columns[1].references.as_ref().unwrap();
                assert_eq!(fk1.table, "users");
                assert_eq!(fk1.column, "id");
                // Third column: product_id with FK to products
                assert_eq!(ct.columns[2].name, "product_id");
                assert!(ct.columns[2].references.is_some());
                let fk2 = ct.columns[2].references.as_ref().unwrap();
                assert_eq!(fk2.table, "products");
                assert_eq!(fk2.column, "id");
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }

    #[test]
    fn test_parse_column_auto_increment_synonyms() {
        let result = parse("CREATE TABLE t1 (id INTEGER AUTOINCREMENT)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[test]
    fn test_parse_show_status() {
        let result = parse("SHOW STATUS");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::ShowStatus => {}
            _ => panic!("Expected SHOW STATUS statement"),
        }
    }

    #[test]
    fn test_parse_show_processlist() {
        let result = parse("SHOW PROCESSLIST");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::ShowProcesslist => {}
            _ => panic!("Expected SHOW PROCESSLIST statement"),
        }
    }

    // ========================================================================
    // KILL Statement Tests (Issue #1135 - MySQL Compatibility)
    // ========================================================================

    #[test]
    fn test_parse_kill_connection() {
        let result = parse("KILL 12345");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Kill(k) => {
                assert_eq!(k.process_id, 12345);
                assert_eq!(k.kill_type, KillType::Connection);
            }
            _ => panic!("Expected KILL statement"),
        }
    }

    #[test]
    fn test_parse_kill_connection_explicit() {
        let result = parse("KILL CONNECTION 12345");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Kill(k) => {
                assert_eq!(k.process_id, 12345);
                assert_eq!(k.kill_type, KillType::Connection);
            }
            _ => panic!("Expected KILL CONNECTION statement"),
        }
    }

    #[test]
    fn test_parse_kill_query() {
        let result = parse("KILL QUERY 12345");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Kill(k) => {
                assert_eq!(k.process_id, 12345);
                assert_eq!(k.kill_type, KillType::Query);
            }
            _ => panic!("Expected KILL QUERY statement"),
        }
    }

    // ========================================================================
    // COPY Statement Tests (Issue #758 - Parquet Import/Export)
    // ========================================================================

    #[test]
    fn test_parse_copy_from_parquet() {
        let result = parse("COPY users FROM 'users.parquet' (FORMAT PARQUET)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Copy(c) => {
                assert_eq!(c.table_name, "users");
                assert!(c.from, "COPY FROM should have from=true");
                assert_eq!(c.path, "users.parquet");
                assert_eq!(c.format, "PARQUET");
            }
            _ => panic!("Expected COPY statement"),
        }
    }

    #[test]
    fn test_parse_copy_to_parquet() {
        let result = parse("COPY users TO 'backup.parquet' (FORMAT PARQUET)");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Copy(c) => {
                assert_eq!(c.table_name, "users");
                assert!(!c.from, "COPY TO should have from=false");
                assert_eq!(c.path, "backup.parquet");
                assert_eq!(c.format, "PARQUET");
            }
            _ => panic!("Expected COPY statement"),
        }
    }

    #[test]
    fn test_parse_copy_from_parquet_without_format() {
        // FORMAT PARQUET should be optional and default to PARQUET
        let result = parse("COPY users FROM 'users.parquet'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Copy(c) => {
                assert_eq!(c.table_name, "users");
                assert!(c.from, "COPY FROM should have from=true");
                assert_eq!(c.path, "users.parquet");
                assert_eq!(c.format, "PARQUET", "Default format should be PARQUET");
            }
            _ => panic!("Expected COPY statement"),
        }
    }

    #[test]
    fn test_parse_copy_to_parquet_without_format() {
        let result = parse("COPY products TO 'products.parquet'");
        assert!(result.is_ok(), "Error: {:?}", result.err());
        match result.unwrap() {
            Statement::Copy(c) => {
                assert_eq!(c.table_name, "products");
                assert!(!c.from, "COPY TO should have from=false");
                assert_eq!(c.path, "products.parquet");
                assert_eq!(c.format, "PARQUET");
            }
            _ => panic!("Expected COPY statement"),
        }
    }

    #[test]
    fn test_parse_copy_error_no_table() {
        let result = parse("COPY FROM 'file.parquet'");
        assert!(result.is_err(), "Should fail without table name");
    }

    #[test]
    fn test_parse_copy_error_no_direction() {
        let result = parse("COPY users 'file.parquet'");
        assert!(result.is_err(), "Should fail without FROM/TO");
    }

    #[test]
    fn test_parse_copy_error_no_path() {
        let result = parse("COPY users FROM");
        assert!(result.is_err(), "Should fail without file path");
    }
}

// ============================================================================
// MySQL Compatibility Tests (Issue #897)
// ============================================================================

#[test]
fn test_parse_insert_ignore() {
    let result = parse("INSERT IGNORE INTO users (id, name) VALUES (1, 'Alice')");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Insert(i) => {
            assert!(i.ignore, "Should have ignore flag set");
            assert!(!i.replace, "Should not have replace flag set");
            assert_eq!(i.table, "users");
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_replace_into() {
    let result = parse("REPLACE INTO users (id, name) VALUES (1, 'Alice')");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Insert(i) => {
            assert!(i.replace, "Should have replace flag set");
            assert!(!i.ignore, "Should not have ignore flag set");
            assert_eq!(i.table, "users");
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_insert_ignore_on_duplicate() {
    let result = parse("INSERT IGNORE INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name='Bob'");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Insert(i) => {
            assert!(i.ignore, "Should have ignore flag set");
            assert!(i.on_duplicate.is_some(), "Should have on_duplicate");
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_insert_with_set_ignore() {
    let result = parse("INSERT IGNORE INTO users SET id=1, name='Alice'");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Insert(i) => {
            assert!(i.ignore, "Should have ignore flag set");
            assert_eq!(i.table, "users");
            assert_eq!(i.columns.len(), 2);
        }
        _ => panic!("Expected INSERT statement"),
    }
}

#[test]
fn test_parse_alter_table_add_column() {
    let result = parse("ALTER TABLE users ADD COLUMN email TEXT");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::AlterTable(a) => {
            assert_eq!(a.table, "users");
        }
        _ => panic!("Expected ALTER TABLE statement"),
    }
}

#[test]
fn test_parse_alter_table_drop_column() {
    let result = parse("ALTER TABLE users DROP COLUMN age");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::AlterTable(a) => {
            assert_eq!(a.table, "users");
        }
        _ => panic!("Expected ALTER TABLE statement"),
    }
}

#[test]
fn test_parse_create_index() {
    let result = parse("CREATE INDEX idx_name ON users (id)");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateIndex(i) => {
            assert_eq!(i.name, "idx_name");
            assert_eq!(i.table, "users");
            assert!(!i.unique);
        }
        _ => panic!("Expected CREATE INDEX statement"),
    }
}

#[test]
fn test_parse_create_unique_index() {
    let result = parse("CREATE UNIQUE INDEX idx_unique ON users (email)");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateIndex(i) => {
            assert_eq!(i.name, "idx_unique");
            assert_eq!(i.table, "users");
            assert!(i.unique);
        }
        _ => panic!("Expected CREATE INDEX statement"),
    }
}

#[test]
fn test_parse_create_view() {
    let result = parse("CREATE VIEW active_users AS SELECT * FROM users WHERE active = true");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateView(v) => {
            assert_eq!(v.name, "active_users");
            assert!(!v.query.is_empty());
        }
        _ => panic!("Expected CREATE VIEW statement"),
    }
}

#[test]
fn test_parse_transaction_begin() {
    let result = parse("BEGIN");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Transaction(t) => {
            assert_eq!(t.command, TransactionCommand::Begin);
        }
        _ => panic!("Expected Transaction statement"),
    }
}

#[test]
fn test_parse_transaction_commit() {
    let result = parse("COMMIT");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Transaction(t) => {
            assert_eq!(t.command, TransactionCommand::Commit);
        }
        _ => panic!("Expected Transaction statement"),
    }
}

#[test]
fn test_parse_transaction_rollback() {
    let result = parse("ROLLBACK");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Transaction(t) => {
            assert_eq!(t.command, TransactionCommand::Rollback);
        }
        _ => panic!("Expected Transaction statement"),
    }
}

#[test]
fn test_parse_show_status() {
    let result = parse("SHOW STATUS");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::ShowStatus => {}
        _ => panic!("Expected SHOW STATUS statement"),
    }
}

#[test]
fn test_parse_show_processlist() {
    let result = parse("SHOW PROCESSLIST");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::ShowProcesslist => {}
        _ => panic!("Expected SHOW PROCESSLIST statement"),
    }
}

#[test]
fn test_parse_kill() {
    let result = parse("KILL 123");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 123);
        }
        _ => panic!("Expected KILL statement"),
    }
}

#[test]
fn test_parse_kill_connection() {
    let result = parse("KILL CONNECTION 456");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 456);
        }
        _ => panic!("Expected KILL statement"),
    }
}

#[test]
fn test_parse_kill_query() {
    let result = parse("KILL QUERY 789");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Kill(k) => {
            assert_eq!(k.process_id, 789);
        }
        _ => panic!("Expected KILL statement"),
    }
}

#[test]
fn test_parse_truncate() {
    let result = parse("TRUNCATE TABLE users");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Truncate(t) => {
            assert_eq!(t.table_name, "users");
        }
        _ => panic!("Expected TRUNCATE statement"),
    }
}

#[test]
fn test_parse_delete_with_limit() {
    let result = parse("DELETE FROM users WHERE id > 10");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Delete(d) => {
            assert_eq!(d.table, "users");
            assert!(d.where_clause.is_some());
        }
        _ => panic!("Expected DELETE statement"),
    }
}

#[test]
fn test_parse_select_with_order_by() {
    let result = parse("SELECT * FROM users ORDER BY name ASC");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "users");
            assert!(s.order_by.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_with_group_by() {
    let result = parse("SELECT department, COUNT(*) FROM employees GROUP BY department");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Select(s) => {
            assert_eq!(s.table, "employees");
            assert!(s.group_by.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_with_having() {
    let result =
        parse("SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) > 5");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::Select(s) => {
            assert!(s.group_by.is_some());
            assert!(s.having.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_union() {
    let result = parse("SELECT id FROM users UNION SELECT id FROM admins");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::SetOperation(op) => {
            assert_eq!(op.op_type, SetOperationType::Union);
        }
        _ => panic!("Expected SetOperation statement"),
    }
}

#[test]
fn test_parse_union_all() {
    let result = parse("SELECT id FROM users UNION ALL SELECT id FROM admins");
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::SetOperation(op) => {
            assert_eq!(op.op_type, SetOperationType::UnionAll);
        }
        _ => panic!("Expected SetOperation statement"),
    }
}

// Stored Procedure Control Flow Tests

#[test]
fn test_parse_procedure_with_if() {
    let sql = "CREATE PROCEDURE test_if(x INT) BEGIN IF x > 0 THEN SELECT 1; END IF; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert_eq!(proc.name, "test_if");
            assert_eq!(proc.body.len(), 1);
            assert!(matches!(proc.body[0], ProcedureStatement::If { .. }));
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}

#[test]
fn test_parse_procedure_with_while() {
    let sql = "CREATE PROCEDURE test_while() BEGIN WHILE 1 = 1 DO SELECT 1; END WHILE; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert_eq!(proc.name, "test_while");
            assert!(matches!(proc.body[0], ProcedureStatement::While { .. }));
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}

#[test]
fn test_parse_procedure_with_loop_leave() {
    let sql = "CREATE PROCEDURE test_loop() BEGIN LOOP SELECT 1; END LOOP; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert_eq!(proc.name, "test_loop");
            assert!(matches!(proc.body[0], ProcedureStatement::Loop { .. }));
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}

#[test]
fn test_parse_procedure_if_else() {
    let sql = "CREATE PROCEDURE test_if_else(x INT) BEGIN IF x > 0 THEN SELECT 1; ELSE SELECT 2; END IF; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert_eq!(proc.name, "test_if_else");
            match &proc.body[0] {
                ProcedureStatement::If { else_body, .. } => {
                    assert!(!else_body.is_empty());
                }
                _ => panic!("Expected IF statement"),
            }
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}

#[test]
fn test_parse_procedure_with_declare() {
    let sql = "CREATE PROCEDURE test_declare() BEGIN DECLARE x INT DEFAULT 0; SET x = 1; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert!(matches!(proc.body[0], ProcedureStatement::Declare { .. }));
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}

#[test]
fn test_parse_procedure_return() {
    let sql = "CREATE PROCEDURE test_return() BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(result.is_ok(), "Error: {:?}", result.err());
    match result.unwrap() {
        Statement::CreateProcedure(proc) => {
            assert!(matches!(proc.body[0], ProcedureStatement::Return { .. }));
        }
        _ => panic!("Expected CreateProcedure statement"),
    }
}
