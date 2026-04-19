# Parser 模块设计

**版本**: v2.6.0
**模块**: Parser (SQL 解析器)

---

## 一、What (是什么)

Parser 是 SQLRustGo 的 SQL 解析器，负责将 SQL 字符串解析为抽象语法树 (AST)。

## 二、v2.6.0 改进

v2.6.0 增强了以下 SQL 特性:

- 完全相关子查询
- UNION/INTERSECT/EXCEPT 集合操作
- CASE 表达式
- 表表达式

## 三、核心数据结构

```rust
pub enum Statement {
    Select(SelectStmt),
    Insert(InsertStmt),
    Update(UpdateStmt),
    Delete(DeleteStmt),
    // ...
}

pub struct SelectStmt {
    pub distinct: bool,
    pub projections: Vec<Projection>,
    pub from: Option<TableRef>,
    pub where: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
}
```

## 四、相关文档

- [ARCHITECTURE_V2.6.md](../../architecture/ARCHITECTURE_V2.6.md)
- [SQL92_COMPLIANCE.md](../../reports/SQL92_COMPLIANCE.md)

---

*Parser 模块设计 v2.6.0*
