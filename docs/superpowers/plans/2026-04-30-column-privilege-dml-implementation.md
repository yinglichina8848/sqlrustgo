# 列级权限 DML 验证实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 INSERT/UPDATE 语句添加列级权限验证，与 SELECT 保持一致的安全模型

**Architecture:** 在 ExecutionEngine 中添加 `check_column_privileges_for_insert/update()` 方法，复用现有的 `get_authorized_column_indices()` 逻辑，验证用户是否有权访问指定的列

**Tech Stack:** Rust, sqlrustgo_catalog, sqlrustgo_executor

---

## 文件结构

```
src/execution_engine.rs          # 添加 check_* 方法，修改 execute_insert/execute_update
crates/catalog/src/auth.rs      # 可选：添加 SqlError 变体
```

---

## 任务 1: 添加 `check_column_privileges_for_insert` 方法

**Files:**
- Modify: `src/execution_engine.rs:871-970` (在 `execute_insert` 前添加检查)

- [ ] **Step 1: 找到 execute_insert 方法位置**

运行: `rg "fn execute_insert" src/execution_engine.rs`
确认行号

- [ ] **Step 2: 在 execute_insert 开头添加权限检查调用**

```rust
// 在 execute_insert 方法开头 (let table_name = insert.table.clone(); 之后)
let table_name = insert.table.clone();

// 检查列权限
if let Err(e) = self.check_column_privileges_for_insert(&table_name, &insert.columns) {
    return Err(e);
}
```

- [ ] **Step 3: 实现 check_column_privileges_for_insert 方法**

在 `src/execution_engine.rs` 的 `impl ExecutionEngine` 中添加：

```rust
fn check_column_privileges_for_insert(
    &self,
    table: &str,
    columns: &[String],
) -> SqlResult<()> {
    // 如果没有 auth context，跳过检查
    let Some(ref auth_manager) = self.auth_manager else {
        return Ok(());
    };
    let Some(ref current_user) = self.current_user else {
        return Ok(());
    };

    // 空列列表表示 INSERT INTO t VALUES (...) - 无列级检查
    if columns.is_empty() {
        return Ok(());
    }

    let auth = auth_manager.read().unwrap();
    let authorized = auth.get_authorized_columns(current_user, table, Privilege::Write);

    // 检查每一列
    for col in columns {
        let is_authorized = authorized
            .iter()
            .any(|auth_col| auth_col.eq_ignore_ascii_case(col) || auth_col == "*");

        if !is_authorized {
            return Err(format!("Column '{}' not accessible", col)).into());
        }
    }

    Ok(())
}
```

- [ ] **Step 4: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -50`
预期: 无编译错误

- [ ] **Step 5: 提交**

```bash
git add src/execution_engine.rs
git commit -m "feat(execution): add column privilege check for INSERT"
```

---

## 任务 2: 添加 `check_column_privileges_for_update` 方法

**Files:**
- Modify: `src/execution_engine.rs:996-1125` (execute_update 方法)

- [ ] **Step 1: 找到 execute_update 方法位置**

运行: `rg "fn execute_update" src/execution_engine.rs`
确认行号

- [ ] **Step 2: 在 execute_update 开头添加权限检查调用**

```rust
// 在 execute_update 方法中，获取 table_name 后添加检查
let table_name = update.table.clone();

// 检查列权限 - 验证 SET 子句中的所有列
if let Err(e) = self.check_column_privileges_for_update(&table_name, &update.columns) {
    return Err(e);
}
```

- [ ] **Step 3: 实现 check_column_privileges_for_update 方法**

在 `impl ExecutionEngine` 中添加：

```rust
fn check_column_privileges_for_update(
    &self,
    table: &str,
    columns: &[String],
) -> SqlResult<()> {
    // 如果没有 auth context，跳过检查
    let Some(ref auth_manager) = self.auth_manager else {
        return Ok(());
    };
    let Some(ref current_user) = self.current_user else {
        return Ok(());
    };

    // 空列列表不应该发生，但安全处理
    if columns.is_empty() {
        return Ok(());
    }

    let auth = auth_manager.read().unwrap();
    let authorized = auth.get_authorized_columns(current_user, table, Privilege::Write);

    // 检查每一列是否有权限
    for col in columns {
        let is_authorized = authorized
            .iter()
            .any(|auth_col| auth_col.eq_ignore_ascii_case(col) || auth_col == "*");

        if !is_authorized {
            return Err(format!("Column '{}' not accessible", col)).into());
        }
    }

    Ok(())
}
```

- [ ] **Step 4: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -50`
预期: 无编译错误

- [ ] **Step 5: 提交**

```bash
git add src/execution_engine.rs
git commit -m "feat(execution): add column privilege check for UPDATE"
```

---

## 任务 3: 添加测试用例

**Files:**
- Modify: `src/execution_engine.rs` (在 tests 模块中添加)

- [ ] **Step 1: 找到现有测试模块位置**

运行: `rg "#\[cfg\(test\)\]" src/execution_engine.rs`
确认 tests 模块开始位置

- [ ] **Step 2: 添加 INSERT 权限测试**

在 tests 模块中添加：

```rust
#[test]
fn test_insert_column_privilege_denied() {
    use sqlrustgo_catalog::auth::{AuthManager, UserIdentity};
    use sqlrustgo_storage::memory_storage::MemoryStorage;
    use std::sync::Arc;
    use parking_lot::RwLock;

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage.clone());

    engine.execute("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)").unwrap();

    // 设置 auth - 用户只有 id, name 的 Write 权限
    let auth_manager = Arc::new(RwLock::new(AuthManager::new()));
    let identity = UserIdentity::new("alice", "localhost");
    {
        let mut auth = auth_manager.write().unwrap();
        auth.create_user(&identity, "hash").unwrap();
        auth.grant_column_privilege(&identity, Privilege::Write, "users", "id", 0).unwrap();
        auth.grant_column_privilege(&identity, Privilege::Write, "users", "name", 1).unwrap();
        // 注意：没有 grant email 列
    }
    engine.set_auth_context(auth_manager, identity);

    // 尝试 INSERT 无权列
    let result = engine.execute("INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Column 'email' not accessible"));
}
```

- [ ] **Step 3: 添加 UPDATE 权限测试**

```rust
#[test]
fn test_update_column_privilege_denied() {
    use sqlrustgo_catalog::auth::{AuthManager, UserIdentity};
    use sqlrustgo_storage::memory_storage::MemoryStorage;
    use std::sync::Arc;
    use parking_lot::RwLock;

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage.clone());

    engine.execute("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)").unwrap();
    engine.execute("INSERT INTO users VALUES (1, 'Bob', 'bob@example.com')").unwrap();

    // 设置 auth - 用户只有 id, name 的 Write 权限
    let auth_manager = Arc::new(RwLock::new(AuthManager::new()));
    let identity = UserIdentity::new("alice", "localhost");
    {
        let mut auth = auth_manager.write().unwrap();
        auth.create_user(&identity, "hash").unwrap();
        auth.grant_column_privilege(&identity, Privilege::Write, "users", "id", 0).unwrap();
        auth.grant_column_privilege(&identity, Privilege::Write, "users", "name", 1).unwrap();
    }
    engine.set_auth_context(auth_manager, identity);

    // 尝试 UPDATE 无权列
    let result = engine.execute("UPDATE users SET email = 'new@example.com' WHERE id = 1");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Column 'email' not accessible"));
}
```

- [ ] **Step 4: 运行测试验证**

运行: `cargo test --lib execution_engine::tests --all-features 2>&1 | tail -30`
预期: 所有新测试通过

- [ ] **Step 5: 提交**

```bash
git add src/execution_engine.rs
git commit -m "test(execution): add column privilege tests for INSERT/UPDATE"
```

---

## 任务 4: 运行完整验证

- [ ] **Step 1: 运行 clippy**

运行: `cargo clippy --all-features -- -D warnings 2>&1 | tail -20`
预期: 无 warnings

- [ ] **Step 2: 运行格式检查**

运行: `cargo fmt --check --all 2>&1`
预期: 无格式问题

- [ ] **Step 3: 运行所有测试**

运行: `cargo test --all-features 2>&1 | tail -30`
预期: 所有测试通过

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat(execution): implement column-level privilege for INSERT/UPDATE"
```

---

## 自检清单

- [ ] SPEC 覆盖：INSERT 权限检查 ✓, UPDATE 权限检查 ✓
- [ ] 占位符检查：无 TBD/TODO ✓
- [ ] 类型一致性：方法签名在任务 1,2 中一致 ✓
- [ ] 测试覆盖：INSERT 拒绝测试 ✓, UPDATE 拒绝测试 ✓

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-30-column-privilege-dml-implementation.md`**

## 执行选项

**1. Subagent-Driven (推荐)** - 我 dispatch 子代理按任务执行，任务间审查，快速迭代

**2. Inline Execution** - 在当前 session 中使用 executing-plans 执行，批量执行带检查点

**选择哪个？**
