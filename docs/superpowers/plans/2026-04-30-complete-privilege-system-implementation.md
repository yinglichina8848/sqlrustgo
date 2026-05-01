# 完整列级权限系统实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的列级权限系统，包含 DELETE 权限检查、SHOW GRANTS、GRANT OPTION 和 PUBLIC 角色

**Architecture:** 在 AuthManager 中添加表级权限检查和角色管理，在 ExecutionEngine 中集成 DELETE 权限检查和 SHOW GRANTS 执行

**Tech Stack:** Rust, sqlrustgo_catalog, sqlrustgo_executor

---

## 文件结构

```
crates/catalog/src/auth.rs    # 添加角色管理、GRANT OPTION、SHOW GRANTS
src/execution_engine.rs       # 添加 DELETE 权限检查、SHOW GRANTS 执行
```

---

## Task 1: DELETE 权限检查

**Files:**
- Modify: `src/execution_engine.rs:1209` (execute_delete 方法)
- Modify: `crates/catalog/src/auth.rs` (check_table_privilege 方法)

- [ ] **Step 1: 在 AuthManager 中添加 check_table_privilege 方法**

在 `crates/catalog/src/auth.rs` 的 `impl AuthManager` 中添加：

```rust
pub fn check_table_privilege(
    &self,
    user: &UserIdentity,
    object: &ObjectRef,
    privilege: Privilege,
) -> SqlResult<bool> {
    let grants = self.get_grants_for_user(user);
    for grant in grants {
        if grant.matches(privilege, object) {
            return Ok(true);
        }
    }
    Ok(false)
}
```

- [ ] **Step 2: 在 ExecutionEngine 中添加 check_table_privilege_for_delete 方法**

在 `src/execution_engine.rs` 的 `impl ExecutionEngine` 中添加：

```rust
fn check_table_privilege_for_delete(&self, table: &str) -> SqlResult<()> {
    let Some(ref auth_manager) = self.auth_manager else {
        return Ok(());
    };
    let Some(ref current_user) = self.current_user else {
        return Ok(());
    };

    let auth = auth_manager.read().unwrap();
    let authorized = auth.check_table_privilege(
        current_user,
        &ObjectRef::table(table),
        Privilege::Delete,
    )?;

    if !authorized {
        return Err(SqlError::ExecutionError(format!(
            "DELETE command denied to user '{}'@'{}'",
            current_user.username, current_user.host
        )));
    }

    Ok(())
}
```

- [ ] **Step 3: 在 execute_delete 开头集成权限检查**

在 `src/execution_engine.rs` 的 `execute_delete` 方法开头添加：

```rust
fn execute_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
    // 检查 DELETE 权限
    self.check_table_privilege_for_delete(&delete.table)?;
    // ... 现有逻辑
}
```

- [ ] **Step 4: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -30`
预期: 无编译错误

- [ ] **Step 5: 提交**

```bash
git add src/execution_engine.rs crates/catalog/src/auth.rs
git commit -m "feat(auth): add check_table_privilege method
feat(execution): add DELETE privilege check"
```

---

## Task 2: SHOW GRANTS 命令

**Files:**
- Modify: `crates/catalog/src/auth.rs` (get_all_grants_for_user 方法)
- Modify: `src/execution_engine.rs` (execute_show_grants 方法)

- [ ] **Step 1: 在 AuthManager 中添加 GrantInfo 结构体和方法**

在 `crates/catalog/src/auth.rs` 中添加：

```rust
#[derive(Debug, Clone)]
pub struct GrantInfo {
    pub user: UserIdentity,
    pub privilege: Privilege,
    pub object: ObjectRef,
    pub columns: Option<Vec<String>>,
    pub grant_option: bool,
}

impl AuthManager {
    pub fn get_all_grants_for_user(&self, user: &UserIdentity) -> Vec<GrantInfo> {
        let mut grants = Vec::new();
        if let Some(user_grants) = self.user_privileges.get(user) {
            for grant in user_grants {
                grants.push(GrantInfo {
                    user: user.clone(),
                    privilege: grant.privilege.clone(),
                    object: grant.object.clone(),
                    columns: grant.columns.clone(),
                    grant_option: grant.grant_option,
                });
            }
        }
        grants
    }
}
```

- [ ] **Step 2: 在 ExecutionEngine 中添加 execute_show_grants 方法**

在 `src/execution_engine.rs` 的 `impl ExecutionEngine` 中添加：

```rust
fn execute_show_grants(&self, user: Option<&str>) -> SqlResult<ExecutorResult> {
    let target_user = if let Some(u) = user {
        UserIdentity::new(u, "%")
    } else {
        self.current_user.clone().unwrap()
    };

    let auth = self.auth_manager.as_ref().unwrap().read().unwrap();
    let grants = auth.get_all_grants_for_user(&target_user);

    let rows: Vec<Vec<Value>> = grants
        .iter()
        .map(|g| {
            let mut sql = String::new();
            sql.push_str(&format!("GRANT {} ON ", g.privilege));
            sql.push_str(&format!("{}", g.object));
            if let Some(ref cols) = g.columns {
                sql.push_str(&format!(" ({})", cols.join(", ")));
            }
            sql.push_str(&format!(" TO '{}'@'{}'", g.user.username, g.user.host));
            if g.grant_option {
                sql.push_str(" WITH GRANT OPTION");
            }
            Value::Text(sql)
        })
        .map(|v| vec![v])
        .collect();

    Ok(ExecutorResult::new(rows, rows.len()))
}
```

- [ ] **Step 3: 在 execute_statement 中添加 SHOW GRANTS 分支**

在 `execute_statement` 方法中添加：

```rust
Statement::ShowGrants(user) => self.execute_show_grants(user.as_deref()),
```

- [ ] **Step 4: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -30`
预期: 无编译错误

- [ ] **Step 5: 提交**

```bash
git add crates/catalog/src/auth.rs src/execution_engine.rs
git commit -m "feat(auth): add get_all_grants_for_user method
feat(execution): add SHOW GRANTS command"
```

---

## Task 3: GRANT OPTION 支持

**Files:**
- Modify: `crates/catalog/src/auth.rs` (PrivilegeGrant 结构体, grant_option 追踪)

- [ ] **Step 1: 修改 PrivilegeGrant 添加 grant_option 字段**

找到 `PrivilegeGrant` 结构体定义，添加 `grant_option` 字段：

```rust
pub struct PrivilegeGrant {
    pub privilege: Privilege,
    pub object: ObjectRef,
    pub columns: Option<Vec<String>>,
    pub grant_option: bool,  // 新增字段
}
```

- [ ] **Step 2: 修改 grant_privilege 方法支持 grant_option 参数**

修改 `grant_privilege` 方法签名和实现：

```rust
pub fn grant_privilege(
    &mut self,
    grantor: &UserIdentity,
    grantee: &UserIdentity,
    privilege: Privilege,
    object: &ObjectRef,
    grant_option: bool,
) -> SqlResult<()> {
    // 检查 grantor 是否有 grant_option
    if grant_option && !self.has_grant_option(grantor, privilege, object)? {
        return Err(SqlError::ExecutionError(
            "Access denied: you need GRANT OPTION".to_string(),
        ));
    }

    let grant = PrivilegeGrant {
        privilege,
        object: object.clone(),
        columns: None,
        grant_option,
    };

    self.user_privileges
        .entry(grantee.clone())
        .or_default()
        .push(grant);

    Ok(())
}
```

- [ ] **Step 3: 添加 has_grant_option 方法**

```rust
pub fn has_grant_option(
    &self,
    user: &UserIdentity,
    privilege: Privilege,
    object: &ObjectRef,
) -> SqlResult<bool> {
    let grants = self.get_grants_for_user(user);
    for grant in grants {
        if grant.privilege == privilege && grant.matches_object(object) && grant.grant_option {
            return Ok(true);
        }
    }
    Ok(false)
}
```

- [ ] **Step 4: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -30`
预期: 无编译错误

- [ ] **Step 5: 提交**

```bash
git add crates/catalog/src/auth.rs
git commit -m "feat(auth): add grant_option tracking and has_grant_option method"
```

---

## Task 4: PUBLIC 角色

**Files:**
- Modify: `crates/catalog/src/auth.rs` (PUBLIC 角色创建和分配)

- [ ] **Step 1: 在 AuthManager::new 中创建 PUBLIC 角色**

找到 `AuthManager::new` 方法，添加：

```rust
pub fn new() -> Self {
    let mut auth = Self { ... };
    auth.roles.insert(
        "PUBLIC".to_string(),
        Role {
            name: "PUBLIC".to_string(),
            parent: None,
            privileges: Vec::new(),
        },
    );
    auth
}
```

- [ ] **Step 2: 修改 create_user 方法自动分配 PUBLIC 角色**

```rust
pub fn create_user(
    &mut self,
    identity: &UserIdentity,
    password_hash: &str,
) -> SqlResult<()> {
    let user = UserAuthInfo {
        identity: identity.clone(),
        password_hash: password_hash.to_string(),
        is_active: true,
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
    };

    self.users.insert(identity.clone(), user);
    self.assign_role(identity, "PUBLIC")?;
    Ok(())
}
```

- [ ] **Step 3: 修改 effective_permissions 合并 PUBLIC 权限**

```rust
pub fn effective_permissions(&self, user: &UserIdentity) -> Vec<PrivilegeGrant> {
    let mut grants = self.get_direct_grants(user);

    // 合并 PUBLIC 角色权限
    if let Some(public_role) = self.roles.get("PUBLIC") {
        grants.extend(public_role.privileges.clone());
    }

    grants
}
```

- [ ] **Step 4: 修改 check_privilege 使用 effective_permissions**

```rust
pub fn check_privilege(
    &self,
    user: &UserIdentity,
    object: &ObjectRef,
    privilege: Privilege,
) -> SqlResult<bool> {
    let grants = self.effective_permissions(user);
    for grant in grants {
        if grant.matches(privilege, object) {
            return Ok(true);
        }
    }
    Ok(false)
}
```

- [ ] **Step 5: 验证编译**

运行: `cargo build --all-features -p sqlrustgo 2>&1 | head -30`
预期: 无编译错误

- [ ] **Step 6: 提交**

```bash
git add crates/catalog/src/auth.rs
git commit -m "feat(auth): add PUBLIC role and auto-assignment to new users"
```

---

## Task 5: 测试用例

**Files:**
- Modify: `crates/catalog/src/auth.rs` (tests 模块)
- Modify: `src/execution_engine.rs` (tests 模块)

- [ ] **Step 1: 添加 DELETE 权限测试**

```rust
#[test]
fn test_delete_without_privilege() {
    let mut auth = AuthManager::new();
    let alice = UserIdentity::new("alice", "localhost");
    auth.create_user(&alice, "hash").unwrap();

    // alice 没有 DELETE 权限
    let result = auth.check_table_privilege(&alice, &ObjectRef::table("users"), Privilege::Delete);
    assert!(!result.unwrap());
}
```

- [ ] **Step 2: 添加 SHOW GRANTS 测试**

```rust
#[test]
fn test_show_grants() {
    let mut auth = AuthManager::new();
    let alice = UserIdentity::new("alice", "localhost");
    auth.create_user(&alice, "hash").unwrap();

    auth.grant_privilege(&alice, &alice, Privilege::Select, &ObjectRef::table("users"), false)
        .unwrap();

    let grants = auth.get_all_grants_for_user(&alice);
    assert!(grants.iter().any(|g| g.privilege == Privilege::Select));
}
```

- [ ] **Step 3: 添加 GRANT OPTION 测试**

```rust
#[test]
fn test_grant_option() {
    let mut auth = AuthManager::new();
    let alice = UserIdentity::new("alice", "localhost");
    let bob = UserIdentity::new("bob", "localhost");
    auth.create_user(&alice, "hash").unwrap();
    auth.create_user(&bob, "hash").unwrap();

    // alice 授予 bob SELECT WITH GRANT OPTION
    auth.grant_privilege(&alice, &bob, Privilege::Select, &ObjectRef::table("users"), true)
        .unwrap();

    // bob 检查有 grant_option
    let has_go = auth.has_grant_option(&bob, Privilege::Select, &ObjectRef::table("users"))
        .unwrap();
    assert!(has_go);
}
```

- [ ] **Step 4: 添加 PUBLIC 角色测试**

```rust
#[test]
fn test_public_role() {
    let mut auth = AuthManager::new();
    let alice = UserIdentity::new("alice", "localhost");
    auth.create_user(&alice, "hash").unwrap();

    // 授予 PUBLIC SELECT ON users
    auth.grant_role_privilege("PUBLIC", Privilege::Select, &ObjectRef::table("users"))
        .unwrap();

    // alice 应该继承 PUBLIC 权限
    let result = auth.check_privilege(&alice, &ObjectRef::table("users"), Privilege::Select)
        .unwrap();
    assert!(result);
}
```

- [ ] **Step 5: 运行所有测试**

运行: `cargo test --all-features 2>&1 | tail -30`
预期: 所有测试通过

- [ ] **Step 6: 提交**

```bash
git add crates/catalog/src/auth.rs src/execution_engine.rs
git commit -m "test: add privilege system tests"
```

---

## Task 6: 完整验证

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
git commit -m "feat: implement complete privilege system with DELETE, SHOW GRANTS, GRANT OPTION, and PUBLIC role"
```

---

## 自检清单

- [ ] SPEC 覆盖：DELETE 权限检查 ✓, SHOW GRANTS ✓, GRANT OPTION ✓, PUBLIC 角色 ✓
- [ ] 占位符检查：无 TBD/TODO ✓
- [ ] 类型一致性：方法签名一致 ✓
- [ ] 测试覆盖：DELETE 测试 ✓, SHOW GRANTS 测试 ✓, GRANT OPTION 测试 ✓, PUBLIC 测试 ✓

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-30-complete-privilege-system-implementation.md`**

## 执行选项

**1. Subagent-Driven (推荐)** - 我 dispatch 子代理按任务执行，任务间审查，快速迭代

**2. Inline Execution** - 在当前 session 中使用 executing-plans 执行，批量执行带检查点

**选择哪个？**
