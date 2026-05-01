# SQL-92 功能缺失 - SQL Corpus 测试失败

## 概述

SQL Corpus 测试显示 SQL-92 功能仍有缺失，需要完善。

## 测试结果

| 指标 | 数值 |
|------|------|
| 总测试用例 | 59 |
| 通过 | 55 |
| 失败 | 4 |
| 通过率 | 93.2% |

## 失败的测试用例

### 1. HAVING 子句解析失败

**测试**: `having_clause`
**错误**: `Parse error: "Expected expression"`

**原因**: HAVING 子句解析未实现

**相关代码**: Parser 模块

**修复建议**:
- 实现 HAVING 子句解析
- 在 SELECT 语句中添加 HAVING 支持

### 2. 外键约束解析失败 (3个)

**测试**:
- `fk_insert_valid`
- `fk_insert_invalid`
- `fk_delete_parent`

**错误**: `Parse error: "Expected LParen, got Identifier("user_id")"`

**原因**: CREATE TABLE 语句中的 FOREIGN KEY 语法解析未完全实现

**相关代码**: Parser 模块

**修复建议**:
- 实现完整的 FOREIGN KEY 语法解析
- 支持 `FOREIGN KEY (column) REFERENCES table(column)` 格式

## 需要实现的功能

### P0 - 必须修复

| 功能 | 描述 | 影响测试 |
|------|------|----------|
| HAVING 子句 | SELECT ... GROUP BY ... HAVING | having_clause |
| 外键约束 | CREATE TABLE 中的 FOREIGN KEY | fk_insert_valid, fk_insert_invalid, fk_delete_parent |

## 修复计划

### 步骤 1: 修复 HAVING 子句

```rust
// 在 parser.rs 中添加
fn parse_having_clause(&mut self) -> ParseResult<Option<Expression>> {
    // 解析 HAVING 后的表达式
}
```

### 步骤 2: 修复外键约束

```rust
// 在 parser.rs 中添加
fn parse_foreign_key(&mut self) -> ParseResult<ForeignKeyConstraint> {
    // 解析 FOREIGN KEY (col) REFERENCES table(col)
}
```

## 验证

修复后运行:

```bash
cargo test -p sqlrustgo-sql-corpus
```

目标: 100% 通过 (59/59)

## 相关 Issue

- Related to #1498: SQL-92 语法支持
- Related to #1497: 功能集成