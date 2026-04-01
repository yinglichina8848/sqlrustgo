# SQLRustGo v2.0.0 Parser 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-parser

---

## 1. 模块概述

Parser 模块负责 SQL 语句的词法分析、语法解析和 AST 构建。

## 2. 核心组件

### 2.1 Lexer (词法分析器)

```rust
pub struct Lexer {
    source: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self;
    pub fn tokenize(&mut self) -> Result<Vec<Token>>;
    pub fn next_token(&mut self) -> Result<Token>;
    pub fn peek_token(&mut self) -> Result<Token>;
}
```

### 2.2 Token 类型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    SELECT,
    FROM,
    WHERE,
    INSERT,
    UPDATE,
    DELETE,
    CREATE,
    DROP,
    INTO,
    TABLE,
    INDEX,
    DATABASE,
    VALUES,
    SET,
    AND,
    OR,
    NOT,
    IS,
    NULL,
    TRUE,
    FALSE,
    INTEGER,
    BIGINT,
    VARCHAR,
    TEXT,
    FLOAT,
    DOUBLE,
    BOOLEAN,
    DATE,
    TIMESTAMP,
    INTERVAL,
    IDENTIFIER(String),
    STRING(String),
    NUMBER(String),
    OPERATOR(String),
    PUNCTUATION(char),
    EOF,
}
```

### 2.3 Parser (语法分析器)

```rust
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self;
    pub fn parse(&mut self) -> Result<Vec<Statement>>;
    pub fn parse_statement(&mut self) -> Result<Statement>;
    pub fn parse_select(&mut self) -> Result<SelectStatement>;
    pub fn parse_insert(&mut self) -> Result<InsertStatement>;
    pub fn parse_update(&mut self) -> Result<UpdateStatement>;
    pub fn parse_delete(&mut self) -> Result<DeleteStatement>;
    pub fn parse_create_table(&mut self) -> Result<CreateTableStatement>;
    pub fn parse_copy(&mut self) -> Result<CopyStatement>;
}
```

---

## 3. AST 节点

### 3.1 Statement 枚举

```rust
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    CreateIndex(CreateIndexStatement),
    DropIndex(DropIndexStatement),
    TruncateTable(TruncateTableStatement),
    AlterTable(AlterTableStatement),
    Begin(TransactionStatement),
    Commit,
    Rollback,
    Explain(ExplainStatement),
    Copy(CopyStatement),
    ShowTables,
    ShowDatabases,
    ShowColumns,
    Grant(GrantStatement),
    Revoke(RevokeStatement),
    CreateUser(CreateUserStatement),
    DropUser(DropUserStatement),
}
```

### 3.2 SelectStatement

```rust
pub struct SelectStatement {
    pub distinct: bool,
    pub projection: Vec<SelectItem>,
    pub from: Vec<TableWithJoins>,
    pub selection: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderByExpr>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
}

pub enum SelectItem {
    UnnamedExpr(Expr),
    ExprWithAlias { expr: Expr, alias: String },
    Wildcard,
    WildcardQualified(ObjectName),
}

pub struct OrderByExpr {
    pub expr: Expr,
    pub asc: bool,
}
```

### 3.3 CopyStatement (新增)

```rust
pub struct CopyStatement {
    pub table_name: ObjectName,
    pub file_path: String,
    pub format: CopyFormat,
    pub direction: CopyDirection,
    pub options: HashMap<String, String>,
}

pub enum CopyFormat {
    Parquet,
    Csv,
    Json,
}

pub enum CopyDirection {
    From,
    To,
}
```

---

## 4. COPY 语句支持

### 4.1 COPY 语法

```sql
COPY table_name TO 'file.parquet' WITH (FORMAT PARQUET);
COPY table_name FROM 'file.parquet' WITH (FORMAT PARQUET);
COPY orders TO '/data/orders.parquet' WITH (FORMAT PARQUET, COMPRESSION snappy);
```

### 4.2 实现

```rust
pub fn parse_copy(&mut self) -> Result<CopyStatement> {
    self.expect_keyword("COPY")?;
    
    let table_name = self.parse_object_name()?;
    
    let direction = if self.parse_keyword("FROM") {
        CopyDirection::From
    } else if self.parse_keyword("TO") {
        CopyDirection::To
    } else {
        return Err(ParserError::Expected("FROM or TO"));
    };
    
    let file_path = self.parse_literal_string()?;
    
    let mut options = HashMap::new();
    if self.parse_keyword("WITH") {
        self.expect_token(&Token::LParen)?;
        while !self.check_token(&Token::RParen) {
            let key = self.parse_identifier()?;
            self.expect_token(&Token::Eq)?;
            let value = self.parse_value()?;
            options.insert(key, value);
            if !self.check_token(&Token::Comma) {
                break;
            }
            self.next_token()?;
        }
        self.expect_token(&Token::RParen)?;
    }
    
    Ok(CopyStatement {
        table_name,
        file_path,
        format: CopyFormat::Parquet,
        direction,
        options,
    })
}
```

---

## 5. 窗口函数支持

### 5.1 窗口函数语法

```sql
SELECT 
    name,
    department,
    salary,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank,
    SUM(salary) OVER (PARTITION BY department) as dept_total,
    AVG(salary) OVER (ORDER BY hire_date ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as moving_avg
FROM employees;
```

### 5.2 Window 类型

```rust
pub struct Window {
    pub name: Option<String>,
    pub partition_by: Vec<Expr>,
    pub order_by: Vec<OrderByExpr>,
    pub window_frame: Option<WindowFrame>,
}

pub enum WindowFrame {
    Rows(Range),
    Range(Range),
}

pub enum WindowFunction {
    RowNumber,
    Rank,
    DenseRank,
    PercentRank,
    CumeDist,
    Ntile(u32),
    FirstValue(Expr),
    LastValue(Expr),
    Lead(Expr, Option<Expr>),
    Lag(Expr, Option<Expr>),
    Count,
    Sum,
    Avg,
    Min,
    Max,
}
```

---

## 6. 错误处理

```rust
#[derive(Debug, Clone)]
pub enum ParserError {
    UnexpectedToken(Token, Expected),
    Expected(String, Token),
    SyntaxError(String),
    Unsupported(String),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for ParserError {}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
