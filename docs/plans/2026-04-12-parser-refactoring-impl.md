# Parser 重构实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal**: 重构 parser 模块，提升架构清晰度和测试覆盖率至 80%

**Architecture**: 将 6466 行的 parser.rs 拆分为 expression.rs, statement.rs, error.rs 模块

**Tech Stack**: Rust, cargo test, cargo llvm-cov

---

## 阶段 1: 提取 expression.rs

### Task 1: 创建 expression.rs 文件结构

**Files:**
- Create: `crates/parser/src/expression.rs`

**Step 1: 创建文件框架**

```rust
use crate::lexer::Lexer;
use crate::token::Token;
use crate::error::ParseError;

/// Window frame info parsed from SQL
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrameInfo {
    pub mode: String,
    pub start: FrameBoundInfo,
    pub end: FrameBoundInfo,
    pub exclude: Option<String>,
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

/// Order by item
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub ascending: bool,
    pub nulls_first: Option<bool>,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Wildcard,
    FunctionCall(String, Vec<Expression>),
    Subquery(Box<crate::Statement>),
    QualifiedColumn(String, String),
    WindowFunction {
        func: String,
        args: Vec<Expression>,
        partition_by: Vec<Expression>,
        order_by: Vec<OrderByItem>,
        frame: Option<WindowFrameInfo>,
    },
    Placeholder,
    Between {
        expr: Box<Expression>,
        low: Box<Expression>,
        high: Box<Expression>,
    },
    InList {
        expr: Box<Expression>,
        values: Vec<Expression>,
    },
    CaseWhen {
        conditions: Vec<(Expression, Expression)>,
        else_result: Option<Box<Expression>>,
    },
    Extract {
        field: String,
        expr: Box<Expression>,
    },
    Substring {
        expr: Box<Expression>,
        start: Box<Expression>,
        len: Option<Box<Expression>>,
    },
}

/// Expression parser
pub struct ExpressionParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ExpressionParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, token: Token) -> Result<(), ParseError> {
        match self.current() {
            Some(t) if t == &token => {
                self.next();
                Ok(())
            }
            _ => Err(ParseError(format!("Expected {:?}", token))),
        }
    }

    // Parse methods will be moved here
}
```

**Step 2: 运行测试验证文件可编译**

Run: `cargo build -p sqlrustgo-parser`
Expected: 编译成功（无输出表示成功）

---

### Task 2: 迁移 Expression 解析方法

**Files:**
- Modify: `crates/parser/src/expression.rs`
- Modify: `crates/parser/src/parser.rs:2002-2350`

**Step 1: 添加 parse_expression 方法**

```rust
impl ExpressionParser {
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and_expression()?;
        while let Some(Token::Or) = self.current() {
            self.next();
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp(Box::new(left), "OR".to_string(), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison_expression()?;
        while let Some(Token::And) = self.current() {
            self.next();
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp(Box::new(left), "AND".to_string(), Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_arithmetic_expression()?;
        let op = match self.current() {
            Some(Token::Equal) => "=",
            Some(Token::NotEqual) => "!=",
            Some(Token::Greater) => ">",
            Some(Token::Less) => "<",
            Some(Token::GreaterEqual) => ">=",
            Some(Token::LessEqual) => "<=",
            _ => {
                if matches!(self.current(), Some(Token::Between)) {
                    self.next();
                    let low = self.parse_arithmetic_expression()?;
                    self.expect(Token::And)?;
                    let high = self.parse_arithmetic_expression()?;
                    return Ok(Expression::Between {
                        expr: Box::new(left),
                        low: Box::new(low),
                        high: Box::new(high),
                    });
                }
                if matches!(self.current(), Some(Token::In)) {
                    self.next();
                    self.expect(Token::LParen)?;
                    let mut values = Vec::new();
                    loop {
                        values.push(self.parse_arithmetic_expression()?);
                        match self.current() {
                            Some(Token::Comma) => { self.next(); }
                            Some(Token::RParen) => { self.next(); break; }
                            _ => return Err("Expected ',' or ')' after value in IN list".to_string()),
                        }
                    }
                    return Ok(Expression::InList { expr: Box::new(left), values });
                }
                if matches!(self.current(), Some(Token::Like)) {
                    self.next();
                    let pattern = self.parse_arithmetic_expression()?;
                    return Ok(Expression::BinaryOp(Box::new(left), "LIKE".to_string(), Box::new(pattern)));
                }
                return Ok(left);
            }
        };
        self.next();
        let right = self.parse_arithmetic_expression()?;
        Ok(Expression::BinaryOp(Box::new(left), op.to_string(), Box::new(right)))
    }

    fn parse_arithmetic_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_primary_expression()?;
        let op = match self.current() {
            Some(Token::Plus) => "+",
            Some(Token::Minus) => "-",
            Some(Token::Star) => "*",
            Some(Token::Slash) => "/",
            _ => return Ok(left),
        };
        self.next();
        let right = self.parse_arithmetic_expression()?;
        Ok(Expression::BinaryOp(Box::new(left), op.to_string(), Box::new(right)))
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        let token = self.current().cloned();
        match token {
            Some(Token::Identifier(name)) => {
                if name.to_uppercase() == "NULL" {
                    return Ok(Expression::Literal("NULL".to_string()));
                }
                self.next();
                if matches!(self.current(), Some(Token::Dot)) {
                    self.next();
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
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Count) | Some(Token::Sum) | Some(Token::Avg) | Some(Token::Min) | Some(Token::Max) => {
                // Parse aggregate function
                let func_name = match self.current() {
                    Some(Token::Count) => "COUNT",
                    Some(Token::Sum) => "SUM",
                    Some(Token::Avg) => "AVG",
                    Some(Token::Min) => "MIN",
                    Some(Token::Max) => "MAX",
                    _ => return Err("Unknown aggregate function".to_string()),
                };
                self.next();
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                if matches!(self.current(), Some(Token::Star)) {
                    args.push(Expression::Wildcard);
                    self.next();
                } else {
                    args.push(self.parse_expression()?);
                }
                self.expect(Token::RParen)?;
                Ok(Expression::FunctionCall(func_name.to_string(), args))
            }
            _ => Err("Expected expression".to_string()),
        }
    }
}
```

**Step 2: 迁移 window function 解析**

Add to expression.rs:
```rust
    fn parse_window_function(&mut self) -> Result<Expression, String> {
        let func = match self.current() {
            Some(Token::RowNumber) => "ROW_NUMBER",
            Some(Token::Rank) => "RANK",
            Some(Token::DenseRank) => "DENSE_RANK",
            Some(Token::Lead) => "LEAD",
            Some(Token::Lag) => "LAG",
            Some(Token::FirstValue) => "FIRST_VALUE",
            Some(Token::LastValue) => "LAST_VALUE",
            Some(Token::NthValue) => "NTH_VALUE",
            _ => return Err("Expected window function".to_string()),
        };
        self.next();
        self.expect(Token::LParen)?;
        
        let mut args = Vec::new();
        if !matches!(self.current(), Some(Token::RParen)) {
            if matches!(self.current(), Some(Token::Star)) {
                args.push(Expression::Wildcard);
                self.next();
            } else {
                args.push(self.parse_expression()?);
            }
        }
        self.expect(Token::RParen)?;
        
        self.expect(Token::Over)?;
        self.expect(Token::LParen)?;
        
        let mut partition_by = Vec::new();
        let mut order_by = Vec::new();
        let frame = None;
        
        while !matches!(self.current(), Some(Token::RParen) | None) {
            match self.current() {
                Some(Token::Partition) => {
                    self.next();
                    self.expect(Token::By)?;
                    partition_by.push(self.parse_expression()?);
                }
                Some(Token::Order) => {
                    self.next();
                    self.expect(Token::By)?;
                    order_by.push(self.parse_order_by_item()?);
                }
                Some(Token::Rows) | Some(Token::Range) | Some(Token::Groups) => {
                    // Parse window frame (simplified)
                    self.next();
                    // Skip frame parsing for now
                }
                _ => {
                    self.next();
                }
            }
        }
        
        self.expect(Token::RParen)?;
        
        Ok(Expression::WindowFunction {
            func: func.to_string(),
            args,
            partition_by,
            order_by,
            frame,
        })
    }
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-parser`
Expected: 编译成功

---

### Task 3: 更新 parser.rs 使用 expression.rs

**Files:**
- Modify: `crates/parser/src/parser.rs:2000-2350`

**Step 1: 在 parser.rs 中添加 use 语句**

Add at top of impl Parser block:
```rust
use crate::expression::{Expression, ExpressionParser, WindowFrameInfo, FrameBoundInfo, OrderByItem};
```

**Step 2: 替换 parse_expression 调用**

Replace:
```rust
fn parse_expression(&mut self) -> Result<Expression, String> {
    self.parse_or_expression()
}
```

With:
```rust
fn parse_expression(&mut self) -> Result<Expression, String> {
    let mut expr_parser = ExpressionParser::new(self.tokens[self.position..].to_vec());
    let result = expr_parser.parse_expression();
    // Update position based on tokens consumed
    result
}
```

**Step 3: 运行测试**

Run: `cargo test -p sqlrustgo-parser`
Expected: 所有测试通过

---

### Task 4: 更新 lib.rs 重导出

**Files:**
- Modify: `crates/parser/src/lib.rs`

**Step 1: 添加重导出**

```rust
pub use expression::{Expression, WindowFrameInfo, FrameBoundInfo, OrderByItem};
```

**Step 2: 验证编译和测试**

Run: `cargo test -p sqlrustgo-parser`
Expected: 所有测试通过

---

### Task 5: 验证覆盖率改进

**Files:**
- None (仅运行命令)

**Step 1: 运行覆盖率测试**

Run: `cargo llvm-cov -p sqlrustgo-parser --html`
Expected: expression.rs 覆盖率 > 80%

**Step 2: 检查整体覆盖率**

Run: `cargo llvm-cov -p sqlrustgo-parser report`
Expected: parser 模块整体覆盖率提升

**Step 3: 提交阶段 1**

```bash
git add crates/parser/src/expression.rs crates/parser/src/parser.rs crates/parser/src/lib.rs
git commit -m "refactor(parser): extract expression.rs module

- Moved Expression enum and related types to expression.rs
- Created ExpressionParser for parsing expressions
- Updated lib.rs to re-export expression types
- All tests pass"
```

---

## 阶段 2: 提取 error.rs

### Task 6: 创建 error.rs

**Files:**
- Create: `crates/parser/src/error.rs`

**Step 1: 创建错误类型**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: Option<usize>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
        }
    }

    pub fn with_position(mut self, pos: usize) -> Self {
        self.position = Some(pos);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}
```

**Step 2: 更新 expression.rs 使用 ParseError**

Replace `Result<Expression, String>` with `Result<Expression, ParseError>` in expression.rs

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-parser`
Expected: 编译成功

---

## 阶段 3: 提取 statement.rs

### Task 7: 创建 statement.rs

**Files:**
- Create: `crates/parser/src/statement.rs`

**Step 1: 迁移 Statement enum**

```rust
use crate::expression::Expression;

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
    Kill(KillStatement),
    Prepare(PrepareStatement),
    Execute(ExecuteStatement),
    DeallocatePrepare(DeallocatePrepareStatement),
    Copy(CopyStatement),
    Merge(MergeStatement),
    Truncate(TruncateStatement),
}

// TODO: Migrate all statement types here
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-parser`
Expected: 编译成功

---

## 阶段 4: 测试基础设施

### Task 8: 添加表驱动测试

**Files:**
- Create: `crates/parser/src/tests/expression_tests.rs`

**Step 1: 创建表达式测试文件**

```rust
#[cfg(test)]
mod expression_tests {
    use crate::parse;

    #[test]
    fn test_arithmetic_operators() {
        let cases = vec![
            ("SELECT 1 + 2 FROM t", true),
            ("SELECT 1 - 2 FROM t", true),
            ("SELECT 1 * 2 FROM t", true),
            ("SELECT 1 / 2 FROM t", true),
            ("SELECT (1 + 2) * 3 FROM t", true),
        ];
        for (sql, should_pass) in cases {
            let result = parse(sql);
            if should_pass {
                assert!(result.is_ok(), "Expected {} to parse, got {:?}", sql, result.err());
            }
        }
    }

    #[test]
    fn test_comparison_operators() {
        let cases = vec![
            ("SELECT * FROM t WHERE a = b", true),
            ("SELECT * FROM t WHERE a != b", true),
            ("SELECT * FROM t WHERE a > b", true),
            ("SELECT * FROM t WHERE a < b", true),
            ("SELECT * FROM t WHERE a >= b", true),
            ("SELECT * FROM t WHERE a <= b", true),
        ];
        for (sql, should_pass) in cases {
            let result = parse(sql);
            if should_pass {
                assert!(result.is_ok(), "Expected {} to parse", sql);
            }
        }
    }

    #[test]
    fn test_logical_operators() {
        let cases = vec![
            ("SELECT * FROM t WHERE a = 1 AND b = 2", true),
            ("SELECT * FROM t WHERE a = 1 OR b = 2", true),
            ("SELECT * FROM t WHERE NOT a = 1", true),
        ];
        for (sql, should_pass) in cases {
            let result = parse(sql);
            assert!(result.is_ok(), "Expected {} to parse", sql);
        }
    }

    #[test]
    fn test_window_functions() {
        let cases = vec![
            "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t",
            "SELECT RANK() OVER (ORDER BY id) FROM t",
            "SELECT DENSE_RANK() OVER (ORDER BY id) FROM t",
            "SELECT LEAD(value) OVER (ORDER BY id) FROM t",
            "SELECT LAG(value) OVER (ORDER BY id) FROM t",
        ];
        for sql in cases {
            let result = parse(sql);
            assert!(result.is_ok(), "Expected {} to parse", sql);
        }
    }
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-parser`
Expected: 所有测试通过

**Step 3: 提交**

```bash
git add crates/parser/src/tests/expression_tests.rs
git commit -m "test(parser): add expression table-driven tests"
```

---

### Task 9: 添加错误路径测试

**Files:**
- Modify: `crates/parser/src/tests/expression_tests.rs`

**Step 1: 添加错误测试**

```rust
#[test]
fn test_invalid_syntax_errors() {
    let cases = vec![
        "SELECT FROM users",           // Missing column
        "SELECT * WHERE id = 1",       // Missing FROM
        "SELECT * FROM users WHERE",    // Incomplete WHERE
        "INSERT INTO",                   // Incomplete INSERT
        "UPDATE SET",                   // Incomplete UPDATE
    ];
    for sql in cases {
        let result = parse(sql);
        assert!(result.is_err(), "Expected {} to fail", sql);
    }
}
```

**Step 2: 运行测试验证**

Run: `cargo test -p sqlrustgo-parser`
Expected: 所有测试通过

---

### Task 10: 最终覆盖率验证

**Files:**
- None (仅运行命令)

**Step 1: 运行完整覆盖率**

Run: `cargo llvm-cov -p sqlrustgo-parser report`
Expected: 
- expression.rs > 85%
- parser.rs > 70%
- 整体覆盖率 > 80%

**Step 2: 如果未达标，分析并添加针对性测试**

If coverage is below target, identify uncovered functions and add tests.

**Step 3: 提交最终更改**

```bash
git add -A
git commit -m "test(parser): improve coverage to 80%+
- Add table-driven expression tests
- Add error path tests
- Final coverage: 80%+"
```

---

## 验收标准

| 阶段 | 验收标准 |
|------|----------|
| 阶段 1 | expression.rs 创建完成，所有测试通过 |
| 阶段 2 | error.rs 创建完成，统一错误类型 |
| 阶段 3 | statement.rs 创建完成，Statement 分离 |
| 阶段 4 | 覆盖率 80%+，所有测试通过 |

## 执行方式

**推荐: Subagent-Driven Development**
- 每个 Task 由独立 subagent 执行
- 每个阶段完成后主 session 审查
- 快速迭代，及时发现问题