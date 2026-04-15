# Issue #1379: FOREIGN KEY 约束 - Parser + Executor 实现

**日期**: 2026-04-14
**Issue**: #1379
**状态**: 设计阶段

---

## 1. 背景

FOREIGN KEY 约束的 Parser 和 Executor 部分未完成：
1. **Parser**: 只支持列级 `REFERENCES`，不支持表级 FOREIGN KEY 语法
2. **Executor**: CASCADE DELETE/UPDATE 动作未实现

---

## 2. 架构决策

### 2.1 Constraint Naming

**决策**: 必须支持 constraint name

```sql
CREATE TABLE orders (
    id INTEGER,
    user_id INTEGER,
    CONSTRAINT fk_orders_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);
```

**原因**: constraint name 是 catalog-level identity，支持未来 `ALTER TABLE DROP CONSTRAINT` 实现。

### 2.2 CASCADE 实现位置

**决策**: ExecutionEngine 层实现（而非 Storage 层）

| 层 | 职责 |
|----|------|
| Parser | 语法 |
| Planner | 计划 |
| Executor | 语义执行 (CASCADE 属于此层) |
| Storage | tuple read/write |

**原因**: CASCADE 涉及多表扫描、递归删除、事务排序，属于 relational semantics，不是 tuple-level operation。

---

## 3. AST 设计

### 3.1 新增 TableConstraint 结构

```rust
/// 表级约束
#[derive(Debug, Clone, PartialEq)]
pub struct TableConstraint {
    pub name: Option<String>,           // CONSTRAINT name (可选)
    pub constraint_type: ConstraintType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    PrimaryKey { columns: Vec<String> },
    Unique { columns: Vec<String> },
    ForeignKey {
        columns: Vec<String>,           // 本表列
        reference_table: String,         // 引用表
        reference_columns: Vec<String>,  // 引用列
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    },
    Check { condition: String },
}
```

### 3.2 CreateTableStatement 扩展

```rust
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub table_constraints: Vec<TableConstraint>,  // 新增
    pub if_not_exists: bool,
}
```

### 3.3 列级 REFERENCES 保持兼容

列级 `REFERENCES` 语法继续支持，作为语法糖转换到 `TableConstraint::ForeignKey`。

---

## 4. CASCADE 执行策略

### 4.1 执行流程

```
DELETE parent_row
    ↓
catalog.get_referencing_foreign_keys(parent_table)
    ↓
for each referencing FK:
    apply action
        CASCADE → cascade_delete()
        SET NULL → set_null_update()
        RESTRICT → error if rows exist
```

### 4.2 新增 Catalog API

```rust
/// 获取引用指定表的所有外键关系
fn get_referencing_foreign_keys(&self, table: &str) -> Vec<ReferencingForeignKey>;

pub struct ReferencingForeignKey {
    pub child_table: String,
    pub child_columns: Vec<String>,
    pub parent_table: String,
    pub parent_columns: Vec<String>,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}
```

### 4.3 递归保护机制

```rust
fn cascade_delete(
    &mut self,
    table: &str,
    deleted_values: &HashSet<Value>,
    visited: &mut HashSet<String>,  // 防止循环 FK 无限递归
) -> Result<(), SqlError>
```

**保护场景**: self-referencing FK (如 `employees.manager_id → employees.id`)

### 4.4 拓扑排序删除顺序

```
Step 1: collect affected FK relationships
Step 2: topological order by dependency graph
Step 3: execute cascade in order
```

---

## 5. 实现范围

### 5.1 Phase 1 (本 Issue)

| 功能 | 状态 |
|------|------|
| 表级 FK 语法 | 实现 |
| CONSTRAINT name | 实现 |
| 多列 FK | 实现 |
| ON DELETE CASCADE | 实现 |
| ON DELETE SET NULL | 实现 |
| ON DELETE RESTRICT | 实现 |
| ON UPDATE CASCADE | 实现 |
| ON UPDATE SET NULL | 实现 |
| ON UPDATE RESTRICT | 实现 |

### 5.2 Phase 2 (Future)

- DEFERRABLE
- MATCH FULL/PARTIAL
- INITIALLY DEFERRED
- `ALTER TABLE DROP CONSTRAINT`

---

## 6. 测试计划

### 6.1 Parser 测试

- [ ] 表级 FOREIGN KEY 语法解析
- [ ] CONSTRAINT name 解析
- [ ] 多列 FOREIGN KEY 解析
- [ ] 列级 REFERENCES 兼容性

### 6.2 Executor 测试

- [ ] DELETE CASCADE 基础
- [ ] DELETE SET NULL 基础
- [ ] DELETE RESTRICT 基础
- [ ] Self-referencing FK CASCADE
- [ ] 多层 FK CASCADE
- [ ] UPDATE CASCADE
- [ ] 循环 FK 检测

---

## 7. 文件变更

| 文件 | 变更 |
|------|------|
| `crates/parser/src/parser.rs` | TableConstraint AST, CREATE TABLE 扩展 |
| `crates/storage/src/engine.rs` | get_referencing_foreign_keys() |
| `src/lib.rs` | CASCADE 执行逻辑 |
| `tests/integration/foreign_key_test.rs` | 扩展测试 |

---

## 8. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 循环 FK 无限递归 | visited set 保护 |
| 大数据集性能 | 批量操作 + 索引利用 |
| 事务一致性 | 在同一事务内执行所有 CASCADE |
