# SQLRustGo Parser 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-parser

---

## 1. 模块概述

Parser 模块负责 SQL 语句的词法分析和语法解析，将 SQL 文本转换为抽象语法树 (AST)。

### 1.1 模块职责

- 词法分析 (Lexical Analysis)
- 语法解析 (Syntax Parsing)
- AST 构建 (AST Building)
- 词法错误和语法错误报告

### 1.2 模块结构

```
crates/parser/
├── src/
│   ├── lib.rs         # 模块入口和公共接口
│   ├── lexer.rs       # 词法分析器
│   ├── token.rs       # Token 定义
│   └── parser.rs     # 语法解析器
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 Token 定义

**文件**: `crates/parser/src/token.rs`

```uml
@startuml

enum Token {
  + SELECT
  + FROM
  + WHERE
  + INSERT
  + UPDATE
  + DELETE
  + CREATE
  + DROP
  + ALTER
  + TABLE
  + INDEX
  + VIEW
  + INTO
  + VALUES
  + SET
  + AND
  + OR
  + NOT
  + IN
  + LIKE
  + BETWEEN
  + IS
  + NULL
  + JOIN
  + LEFT
  + RIGHT
  + FULL
  + INNER
  + OUTER
  + ON
  + AS
  + ORDER
  + BY
  + GROUP
  + HAVING
  + LIMIT
  + OFFSET
  + UNION
  + INTERSECT
  + EXCEPT
  + ALL
  + DISTINCT
  + PRIMARY
  + KEY
  + FOREIGN
  + REFERENCES
  + UNIQUE
  + CHECK
  + DEFAULT
  + AUTO_INCREMENT
  + INTEGER
  + TEXT
  + FLOAT
  + DOUBLE
  + DATE
  + TIMESTAMP
  + BOOLEAN
  + VARCHAR
  + CHAR
  + DECIMAL
  + BEGIN
  + COMMIT
  + ROLLBACK
  + SAVEPOINT
  + TRANSACTION
  + ISOLATION
  + LEVEL
  + EXPLAIN
  + ANALYZE
  + GRANT
  + REVOKE
  + PRIVILEGES
  + USER
  + INDEXES
  + CONSTRAINTS
  + IDENTIFIER
  + NUMBER
  + STRING
  + OPERATOR
  + EQ
  + NE
  + LT
  + LE
  + GT
  + GE
  + PLUS
  + MINUS
  + STAR
  + SLASH
  + MOD
  + LPAREN
  + RPAREN
  + LBRACKET
  + RBRACKET
  + COMMA
  + DOT
  + SEMICOLON
  + EOF
}

@enduml
```

### 2.2 Lexer 设计

**文件**: `crates/parser/src/lexer.rs`

```uml
@startuml

class Lexer {
  -source: String
  -position: usize
  -line: usize
  -column: usize
  --
  +tokenize(): Vec<Token>
  -next_token(): Token
  -skip_whitespace()
  -read_identifier(): Token
  -read_number(): Token
  -read_string(): Token
  -peek_char(): Option<char>
  -next_char(): Option<char>
}

class Token {
  -type: TokenType
  -lexeme: String
  -literal: Option<Literal>
  -line: usize
  -column: usize
}

Lexer --> Token: produces
Lexer --> TokenType: uses

@enduml
```

### 2.3 Parser 设计

**文件**: `crates/parser/src/parser.rs`

```uml
@startuml

class Parser {
  -tokens: Vec<Token>
  -current: usize
  --
  +parse(): Result<Statement>
  -advance(): Token
  -check(tok: TokenType): bool
  -match(tok: TokenType): bool
  -expect(tok: TokenType): Result<Token>
  -parse_statement(): Result<Statement>
  -parse_select(): Result<Statement>
  -parse_insert(): Result<Statement>
  -parse_update(): Result<Statement>
  -parse_delete(): Result<Statement>
  -parse_create_table(): Result<Statement>
  -parse_drop_table(): Result<Statement>
  -parse_create_index(): Result<Statement>
  -parse_drop_index(): Result<Statement>
  -parse_begin(): Result<Statement>
  -parse_commit(): Result<Statement>
  -parse_rollback(): Result<Statement>
  -parse_explain(): Result<Statement>
  -parse_expression(): Result<Expression>
  -parse_binary_expression(): Result<Expression>
  -parse_unary_expression(): Result<Expression>
  -parse_primary(): Result<Expression>
}

enum Statement {
  Select(SelectStatement)
  Insert(InsertStatement)
  Update(UpdateStatement)
  Delete(DeleteStatement)
  CreateTable(CreateTableStatement)
  DropTable(DropTableStatement)
  CreateIndex(CreateIndexStatement)
  DropIndex(DropIndexStatement)
  CreateView(CreateViewStatement)
  AlterTable(AlterTableStatement)
  Begin(TransactionStatement)
  Commit
  Rollback
  Savepoint(String)
  Explain(ExplainStatement)
  Grant(GrantStatement)
  Revoke(RevokeStatement)
}

Parser --> Statement: produces

@enduml
```

---

## 3. 数据结构设计

### 3.1 Statement 结构

```rust
// Select 语句
pub struct SelectStatement {
    pub projection: Vec<Expression>,
    pub from: Option<TableReference>,
    pub where_clause: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderByExpression>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub is_distinct: bool,
}

// Insert 语句
pub struct InsertStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,
    pub query: Option<Box<Statement>>,
}

// Update 语句
pub struct UpdateStatement {
    pub table_name: String,
    pub set_clauses: Vec<SetClause>,
    pub where_clause: Option<Expression>,
}

// Delete 语句
pub struct DeleteStatement {
    pub table_name: String,
    pub where_clause: Option<Expression>,
}

// Create Table 语句
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub constraints: Vec<TableConstraint>,
    pub if_not_exists: bool,
}

// Column 定义
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Expression>,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub references: Option<ForeignKeyReference>,
}
```

### 3.2 Expression 结构

```uml
@startuml

abstract class Expression {
}

class BinaryExpression {
  -left: Box<Expression>
  -operator: BinaryOperator
  -right: Box<Expression>
}

class UnaryExpression {
  -operator: UnaryOperator
  -operand: Box<Expression>
}

class LiteralExpression {
  -value: Value
}

class ColumnExpression {
  -name: String
  -table: Option<String>
}

class FunctionExpression {
  -name: String
  -arguments: Vec<Expression>
}

class SubqueryExpression {
  -query: Box<Statement>
}

class CaseExpression {
  -conditions: Vec<(Expression, Expression)>
  -else_result: Option<Box<Expression>>
}

enum BinaryOperator {
  + Add
  + Subtract
  + Multiply
  + Divide
  + Modulo
  + And
  + Or
  + Equal
  + NotEqual
  + Less
  + LessEqual
  + Greater
  + GreaterEqual
  + Like
  + In
  + Between
  + Is
}

Expression <|-- BinaryExpression
Expression <|-- UnaryExpression
Expression <|-- LiteralExpression
Expression <|-- ColumnExpression
Expression <|-- FunctionExpression
Expression <|-- SubqueryExpression
Expression <|-- CaseExpression
BinaryExpression --> BinaryOperator

@enduml
```

---

## 4. 算法设计

### 4.1 词法分析流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                        词法分析流程                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Input: "SELECT id, name FROM users WHERE age > 18"                 │
│                                                                      │
│  Start                                                               │
│    │                                                                 │
│    ▼                                                                 │
│  ┌──────────────────┐                                               │
│  │  跳过空白字符     │──yes──▶ 继续跳过                              │
│  │  peek() == ' '   │                                                 │
│  └────────┬─────────┘                                                 │
│           │ no                                                       │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │  peek() 是字母？ │──yes──▶ read_identifier()                    │
│  │  (A-Z, a-z, _)   │                                                 │
│  └────────┬─────────┘                                                 │
│           │ no                                                       │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │  peek() 是数字？ │──yes──▶ read_number()                         │
│  │  (0-9)           │                                                 │
│  └────────┬─────────┘                                                 │
│           │ no                                                       │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │  peek() 是符号？ │──yes──▶ 识别符号 ((){}, etc.)               │
│  └────────┬─────────┘                                                 │
│           │ no                                                       │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │  报错：非法字符  │                                               │
│  └──────────────────┘                                               │
│                                                                      │
│  Output: [SELECT, IDENTIFIER("id"), COMMA, IDENTIFIER("name"),     │
│           FROM, IDENTIFIER("users"), WHERE, IDENTIFIER("age"),     │
│           GT, NUMBER(18), EOF]                                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 语法分析流程

```uml
@startuml

start
:parse();
if (first token) then (SELECT)
  :parse_select();
else if (INSERT)
  :parse_insert();
else if (UPDATE)
  :parse_update();
else if (DELETE)
  :parse_delete();
else if (CREATE)
  :parse_create();
else if (DROP)
  :parse_drop();
else if (BEGIN)
  :parse_begin();
else if (EXPLAIN)
  :parse_explain();
else
  :return Error;
endif

:return Statement;

stop

@enduml
```

### 4.3 递归下降解析

```rust
// 表达式解析 (递归下降)
impl Parser {
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or_expression()
    }
    
    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;
        
        while self.match(TokenType::Or) {
            let right = self.parse_and_expression()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality_expression()?;
        
        while self.match(TokenType::And) {
            let right = self.parse_equality_expression()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // ... 继续解析其他优先级
}
```

---

## 5. 接口设计

### 5.1 公共接口

```rust
// 模块入口
pub use sqlrustgo_parser::{parse, parse_statement};

// 解析入口函数
pub fn parse(sql: &str) -> Result<Statement, SqlError> {
    let mut parser = Parser::new(sql);
    parser.parse_statement()
}

// 解析多条语句
pub fn parse_statements(sql: &str) -> Result<Vec<Statement>, SqlError> {
    let mut parser = Parser::new(sql);
    let mut statements = Vec::new();
    
    while !parser.is_at_end() {
        parser.skip_semicolons();
        if parser.is_at_end() {
            break;
        }
        statements.push(parser.parse_statement()?);
    }
    
    Ok(statements)
}
```

### 5.2 错误处理

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(Token),
    
    #[error("Expected {expected}, found {found}")]
    ExpectedToken { expected: String, found: Token },
    
    #[error("Unterminated string: {0}")]
    UnterminatedString(String),
    
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    
    #[error("Syntax error: {0}")]
    SyntaxError(String),
}
```

---

## 6. 与代码对应检查

### 6.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| Token 定义 | `token.rs` | ✅ 对应 |
| Lexer 实现 | `lexer.rs` | ✅ 对应 |
| Parser 实现 | `parser.rs` | ✅ 对应 |
| Statement 枚举 | `parser.rs` | ✅ 对应 |
| Expression 枚举 | `parser.rs` | ✅ 对应 |

### 6.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| SELECT 解析 | ✅ | ✅ |
| INSERT 解析 | ✅ | ✅ |
| UPDATE 解析 | ✅ | ✅ |
| DELETE 解析 | ✅ | ✅ |
| CREATE TABLE | ✅ | ✅ |
| DROP TABLE | ✅ | ✅ |
| CREATE INDEX | ✅ | ✅ |
| JOIN 解析 | ✅ | ✅ |
| 子查询解析 | ✅ | ✅ |
| 聚合函数 | ✅ | ✅ |
| UNION/INTERSECT/EXCEPT | ✅ | ✅ |
| 事务控制 | ✅ | ✅ |
| EXPLAIN ANALYZE | ✅ | ✅ |

---

## 7. 测试设计

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_select() {
        let result = parse("SELECT id, name FROM users");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_select_with_where() {
        let result = parse("SELECT * FROM users WHERE age > 18");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_join() {
        let result = parse("SELECT * FROM a JOIN b ON a.id = b.id");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_insert() {
        let result = parse("INSERT INTO users (id, name) VALUES (1, 'Alice')");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_create_table() {
        let result = parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)");
        assert!(result.is_ok());
    }
}
```

### 7.2 边界测试

| 测试用例 | 预期结果 |
|----------|----------|
| 空字符串 | Error |
| 仅有空白 | Error |
| 不完整的 SQL | Error |
| 关键字作为列名 | Error |
| 超出范围的数字 | Error |
| 未闭合的引号 | Error |
| 嵌套子查询 | OK |

---

## 8. 性能特征

### 8.1 复杂度分析

| 操作 | 时间复杂度 | 空间复杂度 |
|------|------------|------------|
| 词法分析 | O(n) | O(n) |
| 语法解析 | O(n) | O(n) |
| 表达式解析 | O(n) | O(depth) |

### 8.2 性能优化

- 使用 `&str` 避免字符串复制
- Token 预分配减少重新分配
- 缓存关键字查找结果

---

## 9. 附录

### 9.1 关键字列表

```rust
const KEYWORDS: &[(&str, TokenType)] = &[
    ("SELECT", Token::Select),
    ("FROM", Token::From),
    ("WHERE", Token::Where),
    // ... 共 80+ 关键字
];
```

### 9.2 语法规则摘要

```
statement:
    | select_statement
    | insert_statement
    | update_statement
    | delete_statement
    | create_table_statement
    | drop_table_statement
    | create_index_statement
    | drop_index_statement
    | begin_statement
    | commit_statement
    | rollback_statement
    | explain_statement
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
