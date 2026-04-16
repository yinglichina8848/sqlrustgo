# Issue #1379: 外键约束验证功能实现

**严重程度**: 高
**类型**: 功能实现
**模块**: catalog / executor / parser
**状态**: 待处理
**创建日期**: 2026-04-16

---

## 概述

外键约束功能虽然在 catalog 中定义了数据结构，但在执行器层面未实现验证逻辑，导致 INSERT/UPDATE 时无法检查外键约束。

---

## 问题描述

### 测试失败列表

| 测试 | 描述 | 状态 |
|------|------|------|
| test_fk_table_constraint_single_column | 单列外键 | ❌ 失败 |
| test_fk_table_constraint_multi_column | 复合外键 | ❌ 失败 |
| test_fk_table_constraint_on_delete_cascade | ON DELETE CASCADE | ❌ 失败 |
| test_fk_table_constraint_on_update_set_null | ON UPDATE SET NULL | ❌ 失败 |
| test_unique_table_constraint | 唯一约束 | ❌ 失败 |
| test_primary_key_table_constraint | 主键约束 | ❌ 失败 |

### 失败原因

执行器在 INSERT/UPDATE 时**未调用外键约束验证**。

```rust
// 当前 executor/src/harness.rs 中的实现
table_foreign_keys: None,  // 只是占位，未实现
```

---

## 当前实现状态

### ✅ 已实现

| 组件 | 状态 | 文件 |
|------|------|------|
| 外键解析 | ✅ 已实现 | parser/src |
| ForeignKeyRef 数据结构 | ✅ 已定义 | catalog/src/table.rs |
| ForeignKeyAction 枚举 | ✅ 已定义 | catalog/src/table.rs |
| 表结构 foreign_keys 字段 | ✅ 已添加 | catalog/src/table.rs |

### ❌ 未实现

| 组件 | 状态 | 文件 |
|------|------|------|
| 外键验证逻辑 | ❌ 未实现 | executor |
| INSERT 时外键检查 | ❌ 未实现 | executor |
| UPDATE 时外键检查 | ❌ 未实现 | executor |
| DELETE 时外键检查 (CASCADE/SET NULL) | ❌ 未实现 | executor |
| 主键/唯一约束检查 | ❌ 未实现 | executor |

---

## 需要完成的工作

### Task 1: INSERT 时外键验证

**描述**: 在执行 INSERT 前检查外键约束

**实现位置**: `crates/executor/src/`

**参考代码**:
```rust
impl InsertExecutor {
    pub fn execute(&self, plan: &InsertPlan) -> Result<()> {
        // 1. 获取表的 foreign_keys 定义
        let foreign_keys = self.catalog.get_table_foreign_keys(&plan.table_name)?;
        
        // 2. 对于每个外键列，验证引用表存在对应记录
        for fk in &foreign_keys {
            self.validate_foreign_key(&plan.table_name, fk, &plan.values)?;
        }
        
        // 3. 验证通过后执行插入
        self.inner_execute(plan)
    }
    
    fn validate_foreign_key(&self, table: &str, fk: &ForeignKeyRef, values: &[Value]) -> Result<()> {
        // 查询引用表确认外键存在
        let ref_exists = self.catalog.exists(fk.referenced_table, fk.referenced_column, value)?;
        if !ref_exists {
            return Err(ExecutorError::ForeignKeyViolation(...));
        }
        Ok(())
    }
}
```

**验收标准**:
- [ ] INSERT INTO child_table VALUES (1, 999) 在父表无 id=999 时返回错误
- [ ] 错误消息包含外键约束违反信息

---

### Task 2: UPDATE 时外键验证

**描述**: 在执行 UPDATE 前检查外键约束

**验收标准**:
- [ ] UPDATE child SET user_id = 999 WHERE id = 1 在父表无 id=999 时返回错误

---

### Task 3: DELETE 级联操作

**描述**: 实现 ON DELETE CASCADE/SET NULL/RESTRICT

**验收标准**:
- [ ] DELETE FROM users WHERE id = 1 自动删除关联的 orders (CASCADE)
- [ ] DELETE 时设置子表外键为 NULL (SET NULL)
- [ ] DELETE 时阻止删除有子记录的父记录 (RESTRICT)

---

### Task 4: 主键/唯一约束验证

**描述**: 实现主键和唯一约束检查

**验收标准**:
- [ ] INSERT 重复主键返回错误
- [ ] INSERT 重复唯一键返回错误

---

## 技术参考

### 数据结构

```rust
// crates/catalog/src/table.rs
pub struct ForeignKeyRef {
    pub columns: Vec<String>,           // 子表外键列
    pub referenced_table: String,      // 父表名
    pub referenced_columns: Vec<String>, // 父表主键列
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
}
```

### 相关文件

- 解析器: `crates/parser/src/` (外键语法解析)
- Catalog: `crates/catalog/src/table.rs` (数据结构)
- 执行器: `crates/executor/src/` (验证逻辑 - **待实现**)

---

## 优先级

| Task | 优先级 | 预估工作量 |
|------|--------|----------|
| INSERT 外键验证 | P0 | 2-3 天 |
| UPDATE 外键验证 | P0 | 1 天 |
| DELETE 级联 | P1 | 2 天 |
| 主键/唯一约束 | P1 | 1 天 |

---

## 相关 Issue

- Issue #1480 (覆盖率提升) - 相关测试已添加
- Epic: EPIC-FEATURE_COMPLETION

---

**Issue 创建**: 2026-04-16
**Epic**: EPIC-FEATURE_COMPLETION