# Issue #956 RBAC 权限系统实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现完整的 RBAC 权限系统，包含 CREATE USER/DROP USER/GRANT/REVOKE 语句、SCRAM 认证、mysql.user/mysql.db 系统表

**Architecture:** 混合模式架构 - AuthStore 主存储 + 系统表视图接口。权限检查使用 O(1) HashMap 索引。SCRAM 凭证存储支持未来升级。

**Tech Stack:** Rust, serde, crypto libraries (sha2, pbkdf2, hmac)

---

## 前置准备

1. 切换到 `feature/issue-956-rbac` 分支
2. 阅读 `docs/plans/2026-03-31-rbac-design-v2.md`
3. 阅读现有 `crates/catalog/src/auth.rs` 了解当前实现

---

## Phase 1: Parser - CREATE USER / DROP USER 语法

### Task 1.1: 定义 UserIdentity AST 结构

**Files:**
- Modify: `crates/parser/src/ast.rs`

**Step 1: 添加 UserIdentity 结构到 AST**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UserIdentity {
    pub username: String,
    pub host: String,
}

impl UserIdentity {
    pub fn new(username: &str, host: &str) -> Self {
        Self {
            username: username.to_lowercase(),
            host: host.to_lowercase(),
        }
    }
}
```

**Step 2: 添加 CreateUserStmt 和 DropUserStmt 到 Statement enum**

```rust
pub enum Statement {
    // ... existing variants ...
    CreateUser(CreateUserStmt),
    DropUser(DropUserStmt),
}

#[derive(Debug, Clone)]
pub struct CreateUserStmt {
    pub identities: Vec<UserIdentity>,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct DropUserStmt {
    pub identities: Vec<UserIdentity>,
}
```

---

### Task 1.2: 添加 CREATE USER Token 解析

**Files:**
- Modify: `crates/parser/src/lexer.rs:260-280`
- Modify: `crates/parser/src/token.rs:230-250`
- Modify: `crates/parser/src/parser.rs:480-520`

**Step 1: 在 lexer.rs 添加 CREATE USER 和 DROP USER tokenization**

在 keywords map 中添加：
```rust
"CREATE" => Token::Create,
"USER" => Token::User,
"DROP" => Token::Drop,
```

**Step 2: 在 parser.rs 添加 parse_create_user 方法**

```rust
fn parse_create_user(&mut self) -> Result<Statement, String> {
    self.expect(Token::Create)?;
    self.expect(Token::User)?;

    let mut identities = Vec::new();
    loop {
        self.expect(Token::String(username))?;
        self.expect(Token::At)?;
        self.expect(Token::String(host))?;
        identities.push(UserIdentity::new(&username, &host));

        if !matches!(self.current(), Some(Token::Comma)) {
            break;
        }
        self.next();
    }

    self.expect(Token::Identified)?;
    self.expect(Token::By)?;
    match self.current() {
        Some(Token::String(pwd)) => {
            self.next();
            Ok(Statement::CreateUser(CreateUserStmt {
                identities,
                password: pwd,
            }))
        }
        _ => Err("Expected password after IDENTIFIED BY".to_string()),
    }
}
```

**Step 3: 添加 DROP USER 解析**

类似 parse_create_user，使用 `FROM` 关键字。

---

## Phase 2: AuthStore - UserIdentity + 核心存储

### Task 2.1: 扩展 UserIdentity 实现

**Files:**
- Modify: `crates/catalog/src/auth.rs:1-100`

**Step 1: 添加 Hash derive 和 normalize 方法**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserIdentity {
    pub username: String,
    pub host: String,
}

impl UserIdentity {
    pub fn normalize(&self) -> Self {
        Self {
            username: self.username.to_lowercase(),
            host: self.host.to_lowercase(),
        }
    }
}
```

**Step 2: 实现 Eq + Hash (因为 UserAuthInfo 会存储在 HashMap 中)**

确保 PrivilegeGrant 也实现必要的 trait。

---

### Task 2.2: 扩展 AuthStore 存储结构

**Files:**
- Modify: `crates/catalog/src/auth.rs:100-300`

**Step 1: 修改 privileges 为 HashMap 索引**

```rust
pub struct AuthStore {
    users: HashMap<UserIdentity, UserAuthInfo>,
    privileges: HashMap<UserIdentity, Vec<PrivilegeGrant>>,  // O(1) 权限查询
    next_id: u64,
}
```

**Step 2: 添加 find_exact_user 和 find_wildcard_user 方法**

```rust
impl AuthStore {
    pub fn find_exact_user(&self, identity: &UserIdentity) -> Option<&UserAuthInfo> {
        self.users.get(&identity.normalize())
    }

    pub fn find_wildcard_user(&self, username: &str) -> Option<&UserAuthInfo> {
        self.users.get(&UserIdentity::new(username, "%"))
    }
}
```

---

## Phase 3: SCRAM - credential 生成/验证

### Task 3.1: 定义 ScramCredential 结构

**Files:**
- Modify: `crates/catalog/src/auth.rs:300-400`

**Step 1: 添加 ScramCredential 结构**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScramCredential {
    pub version: u8,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub stored_key: Vec<u8>,
    pub server_key: Vec<u8>,
}

impl ScramCredential {
    pub const CURRENT_VERSION: u8 = 1;
    pub const DEFAULT_ITERATIONS: u32 = 32768;

    pub fn new(password: &str) -> Self {
        let salt = generate_random_salt(32);
        let iterations = Self::DEFAULT_ITERATIONS;
        
        let salted_password = pbkdf2(password, &salt, iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = hash_sha256(&client_key);
        let server_key = hmac_sha256(&salted_password, b"Server Key");

        Self {
            version: Self::CURRENT_VERSION,
            salt,
            iterations,
            stored_key,
            server_key,
        }
    }

    pub fn verify(&self, password: &str) -> bool {
        let salted_password = pbkdf2(password, &self.salt, self.iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = hash_sha256(&client_key);
        constant_time_eq(&stored_key, &self.stored_key)
    }
}
```

**Step 2: 实现辅助函数**

```rust
fn generate_random_salt(len: usize) -> Vec<u8> {
    use std::io::Read;
    let mut buf = vec![0u8; len];
    std::io::Read::read(&mut std::fs::File::open("/dev/urandom").unwrap(), &mut buf).unwrap();
    buf
}

fn pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    // 使用 ring 或 rust-crypto 库实现 PBKDF2-HMAC-SHA256
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    // HMAC-SHA256 实现
}

fn hash_sha256(data: &[u8]) -> Vec<u8> {
    // SHA256 hash 实现
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    // 常数时间比较，防止时序攻击
}
```

---

### Task 3.2: 在 Cargo.toml 添加加密依赖

**Files:**
- Modify: `crates/catalog/Cargo.toml`

**Step 1: 添加依赖**

```toml
[dependencies]
sha2 = "0.10"
hmac = "0.12"
pbkdf2 = { version = "0.12", features = ["simple"] }
rand = "0.8"
```

---

## Phase 4: Executor - CREATE/DROP USER 执行

### Task 4.1: 创建 ddl_executor 模块

**Files:**
- Create: `crates/executor/src/ddl_executor.rs`

**Step 1: 实现 execute_create_user**

```rust
pub fn execute_create_user(
    store: &mut AuthStore,
    stmt: &CreateUserStmt,
    executor_id: u64,
) -> Result<(), ExecutorError> {
    for identity in &stmt.identities {
        let normalized = identity.normalize();
        
        if store.users.contains_key(&normalized) {
            return Err(ExecutorError::UserAlreadyExists {
                user: normalized.username.clone(),
                host: normalized.host.clone(),
            });
        }

        let credential = ScramCredential::new(&stmt.password);
        let user = UserAuthInfo {
            identity: normalized.clone(),
            credential,
            is_active: true,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };
        
        store.users.insert(normalized, user);
    }
    Ok(())
}
```

**Step 2: 实现 execute_drop_user**

类似 CREATE USER，添加 existence check 和 remove 操作。

---

### Task 4.2: 添加 GRANT/REVOKE 执行集成

**Files:**
- Modify: `crates/executor/src/auth_executor.rs`

**Step 1: 实现 execute_grant**

```rust
pub fn execute_grant(
    store: &mut AuthStore,
    stmt: &GrantStatement,
    grantor_id: u64,
) -> Result<(), ExecutorError> {
    let grantee_identity = UserIdentity::new(&stmt.to_user, "localhost");
    
    let grant = PrivilegeGrant {
        id: store.next_id(),
        grantee: grantee_identity.clone(),
        privilege: stmt.privilege,
        object_type: ObjectType::Table,
        object_name: stmt.table.clone(),
        column_name: None,
        granted_by: grantor_id,
        granted_at: current_timestamp(),
    };

    store.privileges
        .entry(grantee_identity)
        .or_insert_with(Vec::new)
        .push(grant);
    
    Ok(())
}
```

---

## Phase 5: System Tables - mysql.user, mysql.db 视图

### Task 5.1: 创建 system_tables 模块

**Files:**
- Create: `crates/catalog/src/system_tables/mod.rs`
- Create: `crates/catalog/src/system_tables/mysql_user.rs`
- Create: `crates/catalog/src/system_tables/mysql_db.rs`

**Step 1: 定义 mysql.user 表结构**

```rust
pub struct MysqlUserTable {
    columns: Vec<ColumnDefinition>,
}

impl MysqlUserTable {
    pub fn rows(&self, auth_store: &AuthStore) -> Vec<Vec<Value>> {
        auth_store
            .users
            .values()
            .map(|user| vec![
                Value::String(user.identity.username.clone()),
                Value::String(user.identity.host.clone()),
                Value::String("caching_sha2_password".to_string()),
                Value::String(encode_scram_verifier(&user.credential)),
                Value::String(if user.is_active { "N" } else { "Y" }.to_string()),
            ])
            .collect()
    }
}
```

**Step 2: 定义 mysql.db 表结构**

类似 mysql.user，从 privileges HashMap 构建。

---

## Phase 6: Integration - 权限检查执行

### Task 6.1: 实现统一权限检查入口

**Files:**
- Modify: `crates/executor/src/auth_executor.rs`

**Step 1: 实现 check_privilege 统一入口**

```rust
pub fn check_privilege(
    store: &AuthStore,
    identity: &UserIdentity,
    object: &DbObject,
    privilege: Privilege,
) -> Result<(), AuthError> {
    let normalized = identity.normalize();

    // 1. 直接检查用户权限
    if let Some(grants) = store.privileges.get(&normalized) {
        if grants.iter().any(|g| g.matches(privilege, object)) {
            return Ok(());
        }
    }

    // 2. 检查 % fallback
    if let Some(wildcard_user) = store.find_wildcard_user(&normalized.username) {
        if let Some(grants) = store.privileges.get(&wildcard_user.identity) {
            if grants.iter().any(|g| g.matches(privilege, object)) {
                return Ok(());
            }
        }
    }

    Err(AuthError::PermissionDenied)
}
```

---

## Phase 7: Tests - 24 场景验收测试

### Task 7.1: 编写 Authentication 测试

**Files:**
- Create: `tests/integration/auth_test.rs`

```rust
#[test]
fn test_auth_password_mismatch() {
    let mut store = AuthStore::new();
    store.create_user("alice", "localhost", "password123");
    
    let result = store.authenticate(&UserIdentity::new("alice", "localhost"), "wrong");
    assert!(result.is_err());
}

#[test]
fn test_auth_user_not_exists() {
    let store = AuthStore::new();
    let result = store.authenticate(&UserIdentity::new("nobody", "localhost"), "password");
    assert!(result.is_err());
}

#[test]
fn test_auth_host_mismatch() {
    let mut store = AuthStore::new();
    store.create_user("alice", "localhost", "password123");
    
    let result = store.authenticate(&UserIdentity::new("alice", "remote"), "password123");
    assert!(result.is_err());
}

#[test]
fn test_auth_wildcard_fallback() {
    let mut store = AuthStore::new();
    store.create_user("alice", "%", "password123");
    
    let result = store.authenticate(&UserIdentity::new("alice", "10.0.0.5"), "password123");
    assert!(result.is_ok());
}
```

---

### Task 7.2: 编写 Authorization 测试

```rust
#[test]
fn test_default_no_privilege() {
    let mut store = AuthStore::new();
    store.create_user("bob", "localhost", "password");
    
    let result = check_privilege(&store, &UserIdentity::new("bob", "localhost"), &DbObject::Table("db1", "table1"), Privilege::Read);
    assert!(result.is_err());
}

#[test]
fn test_grant_and_revoke() {
    let mut store = AuthStore::new();
    store.create_user("alice", "localhost", "password");
    
    store.grant(PrivilegeGrant { privilege: Privilege::Read, object: ... });
    
    assert!(check_privilege(...).is_ok());
    
    store.revoke(...);
    
    assert!(check_privilege(...).is_err());
}
```

---

### Task 7.3: 编写 User Lifecycle 测试

```rust
#[test]
fn test_create_duplicate_fails() {
    let mut store = AuthStore::new();
    store.create_user("alice", "%", "password");
    
    let result = store.create_user("alice", "%", "password2");
    assert!(result.is_err());
}

#[test]
fn test_drop_user_auth_fails() {
    let mut store = AuthStore::new();
    let id = store.create_user("alice", "%", "password").unwrap();
    
    store.drop_user("alice", "%");
    
    assert!(store.authenticate(&UserIdentity::new("alice", "localhost"), "password").is_err());
}
```

---

### Task 7.4: 编写 SCRAM Correctness 测试

```rust
#[test]
fn test_salt_randomness() {
    let cred1 = ScramCredential::new("password");
    let cred2 = ScramCredential::new("password");
    
    assert_ne!(cred1.salt, cred2.salt);
    assert_ne!(cred1.stored_key, cred2.stored_key);
}

#[test]
fn test_stored_key_not_password_hash() {
    let cred = ScramCredential::new("password");
    
    let password_hash = hash_sha256(b"password");
    assert_ne!(cred.stored_key.as_slice(), password_hash.as_slice());
}

#[test]
fn test_iteration_count() {
    let cred = ScramCredential::new("password");
    assert_eq!(cred.iterations, ScramCredential::DEFAULT_ITERATIONS);
}
```

---

## 提交策略

每完成一个 Task 执行一次 commit：

```bash
git add <files>
git commit -m "feat(auth): add UserIdentity struct and normalize method"
```

---

## 验收确认

完成所有 7 个 Phase 后，运行集成测试：

```bash
cargo test --package sqlrustgo-executor auth_test
cargo test --package sqlrustgo-catalog auth_test
```

确保 24 个验收场景全部通过。
