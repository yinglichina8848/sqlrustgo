# Issue #1379: FOREIGN KEY 约束实现

**版本**: v2.5.0  
**状态**: Parser 完成，Executor 未完成  
**日期**: 2026-04-14  
**PR**: https://github.com/minzuuniversity/sqlrustgo/pull/1427

---

## 1. 背景与目标

### 1.1 问题陈述

SQLRustGo 需要实现表级 FOREIGN KEY 约束语法，支持：
- `FOREIGN KEY (col) REFERENCES table(col)` 表级外键
- 复合外键 `FOREIGN KEY (a,b) REFERENCES parent(x,y)`
- `ON DELETE/UPDATE` 动作 (CASCADE, SET NULL, RESTRICT)

### 1.2 架构分层

| 层次 | 状态 | 说明 |
|------|------|------|
| Parser | ✅ 完成 | 语法解析正确 |
| Planner | ⏳ 待处理 | 约束信息传递 |
| Executor | ❌ 未实现 | 约束 enforcement |

---

## 2. 已完成的工作

### 2.1 Parser 层实现

**新增文件**:
- `crates/parser/src/parser.rs` - 核心 TableConstraint 解析

**修改文件**:
- `crates/parser/src/token.rs` - 添加 `Token::Foreign`
- `crates/parser/src/lexer.rs` - 添加 `FOREIGN` 关键字
- `crates/parser/src/lib.rs` - 导出 `TableConstraint`

**实现内容**:

```rust
// TableConstraint 枚举
pub enum TableConstraint {
    PrimaryKey { columns: Vec<Identifier> },
    ForeignKey {
        columns: Vec<Identifier>,
        reference_table: Identifier,
        reference_columns: Vec<Identifier>,
        on_delete: Option<ReferentialAction>,
        on_update: Option<ReferentialAction>,
    },
    Unique { columns: Vec<Identifier> },
}

// CreateTableStatement 扩展
pub struct CreateTableStatement {
    // ... existing fields
    pub constraints: Vec<TableConstraint>,  // 新增
}
```

**支持的语法**:
```sql
-- 单列外键
FOREIGN KEY (user_id) REFERENCES users(id)

-- 复合外键
FOREIGN KEY (order_id, product_id) REFERENCES order_products(order_id, product_id)

-- 带 ON DELETE/UPDATE
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE SET NULL

-- 表级 PRIMARY KEY
PRIMARY KEY (a, b)

-- 表级 UNIQUE
UNIQUE (email)
```

### 2.2 测试验证

**Parser 单元测试**: 315 个全部通过 ✅

**Commits**:
```
fde0f424 fix(parser): correctly parse ON UPDATE SET NULL action
2d8cf0a0 feat(parser): export TableConstraint from lib.rs
5d099f16 feat(parser): add table constraint parsing with ON DELETE/UPDATE support
146428cd feat(parser): integrate table constraint parsing in parse_create_table
c52d811f feat(parser): implement parse_table_constraint and helper methods
f36a4d6e feat(parser): extend CreateTableStatement with constraints field
eea6c1d8 feat(parser): add TableConstraint enum for table-level constraints
```

### 2.3 集成测试 (编译通过，执行失败)

文件: `tests/integration/foreign_key_table_constraint_test.rs`

```rust
// 7 个测试函数
test_fk_table_constraint_single_column          // ❌ 执行失败
test_fk_table_constraint_multi_column           // ❌ 执行失败
test_fk_table_constraint_on_delete_cascade      // ❌ 执行失败
test_fk_table_constraint_on_update_set_null     // ❌ 执行失败
test_primary_key_table_constraint               // ❌ 执行失败
test_fk_and_column_level_fk_together           // ❌ 执行失败
test_unique_table_constraint                    // ❌ 执行失败
```

**失败原因**: Executor 层未实现约束 enforcement

---

## 3. 剩余工作

### 3.1 Executor 层实现

需要实现以下 enforcement 逻辑：

| 约束 | INSERT | UPDATE | DELETE |
|------|--------|--------|--------|
| FK Validation | ✅ 需实现 | ✅ 需实现 | N/A |
| ON DELETE CASCADE | N/A | N/A | ✅ 需实现 |
| ON UPDATE SET NULL | N/A | ✅ 需实现 | N/A |
| PRIMARY KEY uniqueness | ✅ 需实现 | ✅ 需实现 | N/A |
| UNIQUE constraint | ✅ 需实现 | ✅ 需实现 | N/A |

### 3.2 Planner 层修改

需要确保 TableConstraint 信息从 Parser 传递到 Planner：
- `CreateTableStatement.constraints` → `TableInfo.constraints`
- Planner 生成执行计划时包含约束信息

---

## 4. 关键文件

### 4.1 Parser 层 (已完成)

| 文件 | 作用 |
|------|------|
| `crates/parser/src/parser.rs` | TableConstraint 解析逻辑 |
| `crates/parser/src/token.rs` | Token::Foreign 定义 |
| `crates/parser/src/lexer.rs` | FOREIGN 关键字词元化 |
| `crates/parser/src/lib.rs` | 公共导出 |

### 4.2 Executor 层 (待实现)

| 文件 | 作用 |
|------|------|
| `crates/executor/src/insert.rs` | INSERT 时 FK 验证 |
| `crates/executor/src/update.rs` | UPDATE 时 FK 验证 |
| `crates/executor/src/delete.rs` | DELETE 时 CASCADE 处理 |
| `crates/executor/src/constraint.rs` | 新建 - 约束 enforcement |

### 4.3 测试文件

| 文件 | 状态 |
|------|------|
| `tests/integration/foreign_key_table_constraint_test.rs` | 编译通过，执行失败 |

---

## 5. 验证命令

```bash
# Parser 测试 (通过)
cargo test -p sqlrustgo-parser

# 集成测试 (编译通过，执行失败)
cargo test --test foreign_key_table_constraint_test

# 完整构建
cargo build --all-features
```

---

## 6. 下一步行动

1. **Executor FK Validation**: 在 INSERT/UPDATE 时验证外键引用
2. **Executor CASCADE**: 实现 ON DELETE/UPDATE CASCADE 逻辑
3. **Executor SET NULL**: 实现 ON UPDATE SET NULL 逻辑
4. **Planner 集成**: 确保约束信息正确传递

---

**最后更新**: 2026-04-14 22:51
**分支**: `feature/foreign-key-table-constraint`
**目标分支**: `develop/v2.5.0`
