# 完整列级权限系统设计方案

> **日期**: 2026-04-30
> **状态**: 已批准
> **目标**: Issue #37 - 实现完整的列级权限系统

---

## 1. 背景

Issue #25 已实现 SELECT/INSERT/UPDATE 的列级权限检查。本设计扩展到完整实现。

**已有功能：**
- `AuthManager.grant_column_privilege()` - 列权限授予
- `AuthManager.get_authorized_columns()` - 获取授权列
- `ExecutionEngine.get_authorized_column_indices()` - SELECT 列过滤
- `ExecutionEngine.check_column_privileges_for_insert()` - INSERT 列检查
- `ExecutionEngine.check_column_privileges_for_update()` - UPDATE 列检查

**扩展目标：**
- DELETE 表级权限检查
- SHOW GRANTS 命令
- GRANT OPTION 支持
- PUBLIC 角色

---

## 2. 架构设计

### 2.1 组件关系

```
AuthManager
├── check_table_privilege(user, table, Privilege)      [新增]
├── get_grants_for_user(user)                          [新增]
├── grant_option flag tracking                          [新增]
└── PUBLIC role management                             [新增]

ExecutionEngine
├── check_table_privilege_for_delete()                [新增]
├── execute_delete() → 添加 DELETE 权限检查
└── execute_show_grants()                              [新增]
```

### 2.2 核心方法签名

```rust
// AuthManager 新增方法
fn check_table_privilege(
    &self,
    user: &UserIdentity,
    object: &ObjectRef,
    privilege: Privilege,
) -> SqlResult<bool>;

fn get_all_grants_for_user(&self, user: &UserIdentity) -> Vec<GrantInfo>;

fn has_grant_option(
    &self,
    user: &UserIdentity,
    privilege: Privilege,
    object: &ObjectRef,
) -> bool;

// ExecutionEngine 新增方法
fn check_table_privilege_for_delete(&self, table: &str) -> SqlResult<()>;
fn execute_show_grants(&self, user: Option<&str>) -> SqlResult<ExecutorResult>;
```

---

## 3. 功能详细设计

### 3.1 DELETE 权限检查 (P0)

**行为规则：**

| 步骤 | 操作 | 说明 |
|------|------|------|
| 1 | 获取当前用户 | 从 `current_user` 获取 |
| 2 | 检查 DELETE 权限 | `auth.check_table_privilege(user, table, Privilege::Delete)` |
| 3 | 失败则报错 | Error: "DELETE command denied to user" |

**错误信息：**
```
Err("DELETE command denied to user 'alice'@'localhost'")
```

**集成位置：**
```rust
fn execute_delete(&self, delete: &DeleteStatement) -> SqlResult<ExecutorResult> {
    // 检查 DELETE 权限
    self.check_table_privilege_for_delete(&delete.table)?;

    // ... 现有逻辑
}
```

### 3.2 SHOW GRANTS 命令 (P1)

**SQL 语法：**
```sql
SHOW GRANTS FOR 'user'@'host'
SHOW GRANTS FOR CURRENT_USER
```

**输出格式 (MySQL 兼容)：**
```sql
GRANT SELECT, INSERT ON db.users TO 'alice'@'localhost'
GRANT SELECT (id, name) ON db.users TO 'alice'@'localhost'
GRANT DELETE ON db.users TO 'alice'@'localhost' WITH GRANT OPTION
GRANT 'PUBLIC' TO 'alice'@'localhost'
```

**实现：**
```rust
fn execute_show_grants(&self, user: Option<&str>) -> SqlResult<ExecutorResult> {
    let target_user = user
        .map(|u| UserIdentity::parse(u))
        .unwrap_or_else(|| self.current_user.clone().unwrap());

    let grants = auth.get_all_grants_for_user(&target_user);

    let rows: Vec<Vec<Value>> = grants.iter().map(|g| {
        vec![Value::Text(format_grant_sql(g))]
    }).collect();

    Ok(ExecutorResult::new(rows, grants.len()))
}
```

### 3.3 GRANT OPTION 支持 (P1)

**GRANT OPTION 追踪：**

```rust
struct PrivilegeGrant {
    privilege: Privilege,
    object: ObjectRef,
    columns: Option<Vec<String>>,
    grant_option: bool,  // 新增字段
}
```

**行为规则：**

| 场景 | 行为 |
|------|------|
| 授予权限不带 WITH GRANT | `grant_option = false` |
| 授予权限带 WITH GRANT | `grant_option = true` |
| User A 有 grant_option | 可以将权限授予 User B |
| User A 无 grant_option | 不能授予 User B 任何权限 |

**检查逻辑：**
```rust
fn grant_privilege(
    &self,
    grantor: &UserIdentity,
    grantee: &UserIdentity,
    privilege: Privilege,
    object: &ObjectRef,
    grant_option: bool,
) -> SqlResult<()> {
    // 检查 grantor 是否有 WITH GRANT OPTION
    if grant_option && !self.has_grant_option(grantor, privilege, object)? {
        return Err("Access denied: you need GRANT OPTION".into());
    }

    // 执行授予
    // ...
}
```

### 3.4 PUBLIC 角色 (P2)

**创建 PUBLIC 角色：**
```rust
// 在 AuthManager 初始化时创建
fn new() -> Self {
    let mut auth = Self { ... };
    auth.create_role("PUBLIC").unwrap();
    auth
}
```

**PUBLIC 权限分配：**
```rust
// 新用户自动分配 PUBLIC 角色
fn create_user(&mut self, identity: &UserIdentity, password_hash: &str) -> SqlResult<()> {
    // 创建用户
    // ...

    // 分配 PUBLIC 角色
    self.assign_role(identity, "PUBLIC")?;
    Ok(())
}
```

**权限合并：**
```rust
fn effective_permissions(&self, user: &UserIdentity) -> Vec<PrivilegeGrant> {
    let mut grants = self.get_direct_grants(user);

    // 合并 PUBLIC 角色权限
    let public_grants = self.get_role_grants("PUBLIC");
    grants.extend(public_grants);

    grants
}
```

---

## 4. 错误处理

| 场景 | 错误信息 |
|------|----------|
| DELETE 无权限 | `DELETE command denied to user 'user'@'host'` |
| SELECT 无权限 | `SELECT command denied to user 'user'@'host'` |
| INSERT 无权限 | `INSERT command denied to user 'user'@'host'` |
| UPDATE 无权限 | `UPDATE command denied to user 'user'@'host'` |
| 列无权限 | `Column 'column' not accessible` |
| 无 GRANT OPTION | `Access denied: you need GRANT OPTION to grant this privilege` |

---

## 5. 实现位置

| 文件 | 修改内容 |
|------|----------|
| `crates/catalog/src/auth.rs` | 添加 `check_table_privilege`, `get_all_grants_for_user`, `has_grant_option`, PUBLIC 角色 |
| `src/execution_engine.rs` | 添加 `check_table_privilege_for_delete`, `execute_show_grants`, 修改 `execute_delete` |

---

## 6. 测试用例

### 6.1 DELETE 权限测试

```rust
#[test]
fn test_delete_without_privilege() {
    // 用户没有 DELETE 权限
    let result = engine.execute("DELETE FROM users WHERE id = 1");
    assert!(result.is_err());
    assert!(err.to_string().contains("DELETE command denied"));
}
```

### 6.2 SHOW GRANTS 测试

```rust
#[test]
fn test_show_grants() {
    // 授予权限
    auth.grant_privilege(&alice, Privilege::Select, ObjectRef::table("users"));
    auth.grant_column_privilege(&alice, Privilege::Select, "users", "id", 0);

    let result = engine.execute("SHOW GRANTS FOR 'alice'@'localhost'");
    assert!(result.is_ok());
    let rows = result.unwrap().rows;
    assert!(rows.iter().any(|r| r[0].to_string().contains("SELECT")));
}
```

### 6.3 GRANT OPTION 测试

```rust
#[test]
fn test_grant_with_option() {
    // alice 有 SELECT WITH GRANT OPTION
    auth.grant_privilege(&alice, Privilege::Select, ObjectRef::table("users"), true);

    // alice 可以将 SELECT 授予 bob
    auth.grant_privilege_as(&alice, &bob, Privilege::Select, ObjectRef::table("users"));
    assert!(auth.check_privilege(&bob, ObjectRef::table("users"), Privilege::Select).is_ok());
}
```

### 6.4 PUBLIC 角色测试

```rust
#[test]
fn test_public_role() {
    // 授予 PUBLIC 角色 SELECT ON users
    auth.grant_role(&public, "users", Privilege::Select, ObjectRef::table("users"));

    // bob 自动继承 PUBLIC 权限
    assert!(auth.check_privilege(&bob, ObjectRef::table("users"), Privilege::Select).is_ok());
}
```

---

## 7. 不在范围内

- 其他 DDL 语句 (CREATE/ALTER/DROP) 的列级权限
- 角色继承链 (role1 → role2 → role3)
- LDAP/外部认证集成

---

## 8. 变更历史

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-04-30 | 1.0 | 初始版本 |

---

**文档状态**: 已批准
**批准人**: yinglichina8848
