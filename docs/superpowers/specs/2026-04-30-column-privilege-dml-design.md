# 列级权限 DML 验证设计方案

> **日期**: 2026-04-30
> **状态**: 已批准
> **目标**: Issue #25 扩展 - INSERT/UPDATE 列级权限验证

---

## 1. 背景

Issue #25 原本仅要求 SELECT 列级权限过滤（已在 PR #29 中实现）。本设计将范围扩展到 INSERT/UPDATE/DELETE 语句，提供完整的列级权限验证。

**已有功能：**
- `AuthManager.grant_column_privilege()` - 列权限授予
- `AuthManager.get_authorized_columns()` - 获取授权列
- `ExecutionEngine.get_authorized_column_indices()` - SELECT 列过滤

**扩展目标：**
- INSERT 语句列权限验证
- UPDATE 语句列权限验证
- DELETE 语句表级权限检查（无列过滤）

---

## 2. 架构设计

### 2.1 组件关系

```
ExecutionEngine
├── execute_insert()     → check_column_privileges_for_insert()
├── execute_update()     → check_column_privileges_for_update()
├── execute_delete()     → 表级权限检查（无变化）
└── get_authorized_column_indices()  ← 已存在，复用
```

### 2.2 核心方法签名

```rust
/// 检查 INSERT 语句的列权限
/// 验证所有目标列都有 Write 权限
fn check_column_privileges_for_insert(
    &self,
    table: &str,
    columns: &[String],
) -> SqlResult<()> {
    // 1. 获取用户授权的可写列
    // 2. 检查 requested_columns 是否是授权列的子集
    // 3. 如果有任何列无权限，返回 Error 1143
}

/// 检查 UPDATE 语句的列权限
/// 验证 SET 子句中的所有目标列都有 Write 权限
fn check_column_privileges_for_update(
    &self,
    table: &str,
    columns: &[String],
) -> SqlResult<()> {
    // 1. 获取用户授权的可写列
    // 2. 检查 UPDATE 涉及的列是否是授权列的子集
    // 3. 如果无任何可写列，返回 Error 1142（必须至少有一列可写）
}
```

---

## 3. 错误处理

### 3.1 MySQL 兼容错误码

| 场景 | MySQL 错误码 | 错误信息 |
|------|-------------|----------|
| SELECT 列无权限 | 1143 | `Column 'x' not accessible` |
| INSERT 列无权限 | 1143 | `Column 'x' not accessible` |
| UPDATE 列无权限 | 1143 | `Column 'x' not accessible` |
| 无表级权限 | 1142 | `SELECT/INSERT/UPDATE/DELETE command denied to user` |

### 3.2 错误信息格式

```rust
// Error 1143 - 列无权限
Err("Column '{}' not accessible".format(column_name))

// Error 1142 - 表无权限
Err("{} command denied to user '{}'".format(operation, username))
```

---

## 4. 行为规则

### 4.1 INSERT 验证

| 步骤 | 操作 | 说明 |
|------|------|------|
| 1 | 解析 INSERT 列名 | 从 `insert.columns` 获取 |
| 2 | 获取用户授权列 | `auth.get_authorized_columns(user, table, Privilege::Write)` |
| 3 | 验证所有列 | 检查 `insert.columns ⊆ authorized_columns` |
| 4 | 失败则报错 | Error 1143 |

**示例：**
```sql
-- 用户只有 (id, name) 列的 Write 权限
INSERT INTO users (id, name, email) VALUES (1, 'a', 'x');
-- Error 1143: Column 'email' not accessible
```

### 4.2 UPDATE 验证

| 步骤 | 操作 | 说明 |
|------|------|------|
| 1 | 解析 UPDATE 列名 | 从 `update.columns` 获取 |
| 2 | 获取用户授权列 | `auth.get_authorized_columns(user, table, Privilege::Write)` |
| 3 | 验证所有列 | 检查 `update.columns ⊆ authorized_columns` |
| 4 | 验证至少一列可写 | `update.columns ∩ authorized_columns ≠ ∅` |
| 5 | 失败则报错 | Error 1143 或 1142 |

**示例：**
```sql
-- 用户只有 (id, name) 列的 Write 权限
UPDATE users SET email = 'x', secret = 'y' WHERE id = 1;
-- Error 1143: Column 'email' not accessible
-- Error 1143: Column 'secret' not accessible
```

### 4.3 DELETE 验证

| 步骤 | 操作 | 说明 |
|------|------|------|
| 1 | 表级权限检查 | 检查用户是否有 DELETE 权限 |
| 2 | 无列过滤 | DELETE 不涉及列级过滤 |

---

## 5. 实现位置

| 文件 | 修改内容 |
|------|----------|
| `src/execution_engine.rs` | 添加 `check_column_privileges_for_insert/update()` |
| `src/execution_engine.rs` | 修改 `execute_insert/execute_update` 调用验证 |
| `crates/catalog/src/auth.rs` | 可选扩展错误码定义 |

---

## 6. 测试用例

### 6.1 INSERT 测试

```rust
#[test]
fn test_insert_column_privilege_allowed() {
    // 用户有 INSERT 权限的所有列 → 成功
}

#[test]
fn test_insert_column_privilege_denied() {
    // INSERT 无权列 → Error 1143
}

#[test]
fn test_insert_all_columns_allowed() {
    // INSERT * (所有列) → 只插入有权限的列
}
```

### 6.2 UPDATE 测试

```rust
#[test]
fn test_update_column_privilege_allowed() {
    // 用户有 UPDATE 权限的所有列 → 成功
}

#[test]
fn test_update_column_privilege_denied() {
    // UPDATE 无权列 → Error 1143
}

#[test]
fn test_update_requires_at_least_one_writable_column() {
    // 用户对所有列无写权限 → Error 1142
}
```

### 6.3 集成测试

```rust
#[test]
fn test_column_filtering_with_auth_integration() {
    // 完整的 auth 流程测试
}
```

---

## 7. 不在范围内

- GRANT OPTION 支持（简化实现）
- MySQL 协议层错误码映射（后续扩展）
- 其他 DDL 语句（CREATE/ALTER/DROP）的列级权限

---

## 8. 变更历史

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-04-30 | 1.0 | 初始版本 |

---

**文档状态**: 已批准
**批准人**: yinglichina8848
