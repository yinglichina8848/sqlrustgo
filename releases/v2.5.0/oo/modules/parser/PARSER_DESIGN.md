# Parser 模块设计

**版本**: v2.5.0
**模块**: Parser (SQL 解析器)

---

## 一、What (是什么)

Parser 是 SQLRustGo 的 SQL 解析器，负责将 SQL 字符串解析为抽象语法树 (AST)，支持完整的 SQL 方言。

## 二、Why (为什么)

- **语法验证**: 确保输入 SQL 符合语法规范
- **语义分析**: 提取查询结构和语义信息
- **错误报告**: 提供友好的错误提示
- **可扩展性**: 支持新增语法和方言

## 三、How (如何实现)

### 3.1 解析器架构

```
SQL String
    │
    ▼
┌─────────────────────────────────────────┐
│              Lexer                       │
├─────────────────────────────────────────┤
│  - Tokenization                         │
│  - Keyword recognition                  │
│  - Position tracking                    │
└─────────────────────────────────────────┘
    │
    ▼ Tokens
┌─────────────────────────────────────────┐
│              Parser                      │
├─────────────────────────────────────────┤
│  - Recursive descent parsing           │
│  - AST construction                    │
│  - Semantic validation                 │
└─────────────────────────────────────────┘
    │
    ▼ AST
┌─────────────────────────────────────────┐
│         AST Validation                   │
├─────────────────────────────────────────┤
│  - Type checking                       │
│  - Name resolution                     │
│  - Constraint validation                │
└─────────────────────────────────────────┘
```

### 3.2 词法分析器

```rust
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: u32,
    column: u32,
}

impl Lexer {
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // 跳过空白
        self.skip_whitespace();

        // 检查结束
        if self.position >= self.input.len() {
            return Ok(Token::EOF);
        }

        // 解析 token
        match self.current_char()? {
            '(' => self.make_token(TokenType::LParen),
            ')' => self.make_token(TokenType::RParen),
            ',' => self.make_token(TokenType::Comma),
            ';' => self.make_token(TokenType::Semicolon),
            '\'' => self.parse_string(),
            '-' => self.parse_number_or_comment(),
            'A'..='Z' | 'a'..='z' | '_' => self.parse_identifier(),
            _ => Err(LexError::UnexpectedChar(self.current_char()?)),
        }
    }
}
```

### 3.3 语法分析器

```rust
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    // SELECT 语句解析
    pub fn parse_select(&mut self) -> Result<Statement, ParseError> {
        // SELECT
        self.expect(TokenType::Select)?;

        // DISTINCT/ALL (可选)
        let distinct = self.consume_if(TokenType::Distinct).is_some();

        // 投影列
        let projections = self.parse_projection_list()?;

        // FROM (可选)
        let from = if self.consume_if(TokenType::From).is_some() {
            Some(self.parse_table_ref()?)
        } else {
            None
        };

        // WHERE (可选)
        let filter = if self.consume_if(TokenType::Where).is_some() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // GROUP BY (可选)
        let group_by = self.parse_group_by()?;

        // HAVING (可选)
        let having = self.parse_having()?;

        Ok(Statement::Select(SelectStmt {
            distinct,
            projections,
            from,
            where: filter,
            group_by,
            having,
        }))
    }
}
```

### 3.4 AST 结构

```rust
pub enum Statement {
    Select(SelectStmt),
    Insert(InsertStmt),
    Update(UpdateStmt),
    Delete(DeleteStmt),
    CreateTable(CreateTableStmt),
    DropTable(DropTableStmt),
    // ...
}

pub struct SelectStmt {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub from: Option<TableRef>,
    pub where: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderByExpr>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
```

## 四、接口设计

### 4.1 公开 API

```rust
impl Parser {
    // 创建解析器
    pub fn new(sql: &str) -> Self;

    // 解析单条语句
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError>;

    // 解析多条语句
    pub fn parse_statements(&mut self) -> Result<Vec<Statement>, ParseError>;

    // 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError>;

    // 解析类型
    pub fn parse_data_type(&mut self) -> Result<DataType, ParseError>;
}
```

### 4.2 错误类型

```rust
pub enum ParseError {
    LexicalError { message: String, position: Position },
    SyntaxError { message: String, expected: Vec<TokenType>, found: Token },
    SemanticError { message: String, node: String },
}
```

## 五、支持的 SQL 特性

### 5.1 DML

| 特性 | 状态 |
|------|------|
| SELECT | ✅ 完整 |
| INSERT | ✅ 完整 |
| UPDATE | ✅ 完整 |
| DELETE | ✅ 完整 |
| MERGE | ⏳ 开发中 |

### 5.2 DDL

| 特性 | 状态 |
|------|------|
| CREATE TABLE | ✅ 完整 |
| DROP TABLE | ✅ 完整 |
| ALTER TABLE | ✅ 部分 |
| CREATE INDEX | ✅ 完整 |
| CREATE VIEW | ⏳ 开发中 |

### 5.3 表达式

| 特性 | 状态 |
|------|------|
| 算术表达式 | ✅ |
| 比较表达式 | ✅ |
| 逻辑表达式 | ✅ |
| 字符串函数 | ✅ |
| 日期函数 | ✅ |
| 聚合函数 | ✅ |
| 窗口函数 | ✅ |
| 子查询 | ✅ |

## 六、性能考虑

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| 词法分析 | O(n) | 线性扫描 |
| 语法分析 | O(n) | LR/LL |
| AST 构建 | O(n) | 线性 |
| 语义分析 | O(n) | 类型检查 |

### 优化策略

1. **增量解析**: 缓存解析结果
2. **流式解析**: 大查询流式处理
3. **预编译**: 常用查询预解析

## 七、相关文档

- [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md) - 整体架构
- [PLANNER_DESIGN.md](./PLANNER_DESIGN.md) - 查询规划

---

*Parser 模块设计 v2.5.0*
