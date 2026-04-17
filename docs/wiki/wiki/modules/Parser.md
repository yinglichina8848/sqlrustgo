---
entity_type: tool
confidence: 100
domains: [sqlrustgo, parser, sql]
last_updated: 2026-04-17
---

# Parser 模块

> SQL 解析器 - 将 SQL 文本转换为抽象语法树 (AST)

## 概述

Parser 模块负责将 SQL 文本解析为结构化的 LogicalPlan，是 SQL 引擎的入口。

## 源码位置

```
crates/parser/
├── src/
│   ├── lib.rs          # 模块入口
│   ├── lexer.rs        # 词法分析
│   ├── parser.rs       # 语法分析
│   └── token.rs       # Token 定义
└── Cargo.toml
```

## 支持的 SQL 特性

### DDL

| 语句 | 状态 | 说明 |
|------|------|------|
| CREATE TABLE | ✅ | 完整支持 |
| ALTER TABLE | ✅ | ADD COLUMN, RENAME TO |
| DROP TABLE | ✅ | |
| CREATE INDEX | ✅ | |
| CREATE VIEW | ✅ | |
| CREATE TRIGGER | ✅ | 框架存在 |
| CREATE PROCEDURE | ✅ | 框架存在 |

### DML

| 语句 | 状态 | 说明 |
|------|------|------|
| SELECT | ✅ | 完整支持 |
| INSERT | ✅ | VALUES + SELECT |
| UPDATE | ✅ | |
| DELETE | ✅ | |
| UPSERT | ✅ | ON DUPLICATE KEY |

### 高级特性

| 特性 | 状态 | 说明 |
|------|------|------|
| JOIN | ✅ | INNER, LEFT, RIGHT, FULL OUTER, SEMI, ANTI |
| 子查询 | ✅ | EXISTS, IN, ALL, ANY, SOME |
| CTE | ✅ | WITH RECURSIVE 支持 |
| Prepared Statement | ✅ | PREPARE/EXECUTE/DEALLOCATE |
| FOREIGN KEY | ✅ | 表级 + 列级 |

## 核心结构

```rust
// Parser 入口
pub fn parse_sql(sql: &str) -> Result<Statement, ParserError>

// 主要 Statement 类型
Statement::Query(Query)           // SELECT
Statement::Insert(Insert)         // INSERT
Statement::Update(Update)         // UPDATE
Statement::Delete(Delete)         // DELETE
Statement::CreateTable(CreateTable)  // CREATE TABLE
Statement::AlterTable(AlterTable)   // ALTER TABLE
```

## 词法分析 (Lexer)

文件: `lexer.rs`

- 关键字识别
- 字面量解析 (字符串、数字)
- 标识符解析
- 操作符解析

## 语法分析 (Parser)

文件: `parser.rs`

使用递归下降解析器 (Recursive Descent Parser)

```rust
// 解析入口
pub fn parse_statement(&mut self) -> Result<Statement, ParserError>

// SELECT 解析
fn parse_select(&mut self) -> Result<Statement, ParserError>

// WHERE 解析
fn parse_where(&mut self) -> Result<Expression, ParserError>
```

## 测试

```bash
cargo test --package sqlrustgo-parser
```

## 已知问题

- 部分复杂 CTE 场景未测试
- 触发器语法解析后未集成到执行层

## 相关文件

- [Planner 模块](./Planner.md) - 接收 Parser 输出
- [Executor 模块](./Executor.md) - 执行解析后的计划

---

*最后更新: 2026-04-17*
