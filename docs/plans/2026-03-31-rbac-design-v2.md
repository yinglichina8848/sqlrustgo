# Issue #956: RBAC 权限系统 - 完整设计方案 (v2)

**Issue**: #956 RBAC 权限系统 - 用户/角色/GRANT
**创建日期**: 2026-03-31
**状态**: 设计完成，待实现
**版本**: v2.0 (整合 5 项微调优化)

---

## 1. 背景与目标

### 1.1 Issue 描述

Issue #956 是 v2.0 Phase 2 的一部分，要求实现完整的 RBAC 权限系统：

- 系统表 `mysql.user`, `mysql.db`
- `CREATE USER`, `DROP USER`, `GRANT`, `REVOKE` 语句
- SCRAM 密码加密 (SCRAM-SHA-256)
- 连接认证

### 1.2 验收标准

**权限隔离，无越权访问** — 必须满足 24 个验收场景（见附录 A）。

---

## 2. 架构决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 系统表实现 | **混合模式** | AuthStore 主存储 + 系统表视图接口，平衡复杂度与未来演进 |
| user@host 支持 | **支持** | exact match + `%` fallback，暂不支持 prefix wildcard |
| SCRAM 实现 | **标准 SCRAM-SHA-256** | credential storage + local verifier，暂不实现 SASL exchange |
| 索引结构 | **HashMap** | O(1) 权限检查，每条 SQL 都执行，必须常数复杂度 |

---

## 3. 核心数据结构

### 3.1 UserIdentity (用户身份)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserIdentity {
    pub username: String,
    pub host: String,  // exact match 或 "%"
}

impl UserIdentity {
    pub fn new(username: &str, host: &str) -> Self {
        Self {
            username: username.to_lowercase(),
            host: host.to_lowercase(),
        }
    }

    pub fn normalize(&self) -> Self {
        Self {
            username: self.username.to_lowercase(),
            host: self.host.to_lowercase(),
        }
    }
}
```

**重要**: 所有 `UserIdentity` 操作必须调用 `normalize()` 确保一致性。

### 3.2 ScramCredential (SCRAM 凭证)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScramCredential {
    pub version: u8,           // 版本号，支持未来升级 SCRAM-SHA-512 等
    pub salt: Vec<u8>,         // 随机 salt
    pub iterations: u32,       // PBKDF2 iterations，默认 32768
    pub stored_key: Vec<u8>,   // HASH(ClientKey)
    pub server_key: Vec<u8>,   // HMAC(SaltedPassword, "Server Key")
}

impl ScramCredential {
    pub const CURRENT_VERSION: u8 = 1;
    pub const DEFAULT_ITERATIONS: u32 = 32768;

    pub fn new(password: &str) -> Self { ... }
    pub fn verify(&self, password: &str) -> bool { ... }
}
```

**重要**: `version` 字段用于未来 SCRAM 升级，避免 catalog migration。

### 3.3 UserAuthInfo (用户认证信息)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthInfo {
    pub identity: UserIdentity,
    pub credential: ScramCredential,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}
```

### 3.4 PrivilegeGrant (权限授予)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Database,
    Table,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Privilege {
    Read,
    Insert,
    Update,
    Delete,
    Alter,
    Drop,
    Create,
    Grant,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeGrant {
    pub id: u64,
    pub grantee: UserIdentity,
    pub privilege: Privilege,
    pub object_type: ObjectType,
    pub object_name: String,
    pub column_name: Option<String>,
    pub granted_by: u64,
    pub granted_at: u64,
}
```

### 3.5 AuthStore (主存储)

```rust
pub struct AuthStore {
    users: HashMap<UserIdentity, UserAuthInfo>,
    privileges: HashMap<UserIdentity, Vec<PrivilegeGrant>>,  // O(1) 权限查询
    next_id: u64,
}
```

**关键优化**: `privileges` 使用 `HashMap<UserIdentity, Vec<PrivilegeGrant>>` 而非 `Vec<PrivilegeGrant>`，确保权限检查为 O(1)。

---

## 4. 模块划分

```
crates/catalog/src/
├── lib.rs                    # Catalog 导出
├── auth.rs                   # AuthStore, SCRAM, 凭证管理
├── system_tables/
│   ├── mysql_user.rs         # mysql.user 视图
│   └── mysql_db.rs           # mysql.db 视图

crates/parser/src/
├── parser.rs                 # CREATE USER, DROP USER, GRANT, REVOKE 语法

crates/executor/src/
├── ddl_executor.rs           # CREATE USER, DROP USER 执行
└── auth_executor.rs          # 权限检查执行

crates/server/src/
└── auth_handler.rs           # 连接认证
```

---

## 5. SQL 语法支持

### 5.1 CREATE USER

```sql
CREATE USER 'username'@'host' IDENTIFIED BY 'password';
CREATE USER 'u1'@'h1', 'u2'@'h2';
```

| 参数 | 说明 |
|------|------|
| `username` | 用户名（小写） |
| `host` | 主机名，支持 exact 和 `%` wildcard |
| `password` | 明文密码，内部生成 SCRAM 凭证 |

### 5.2 DROP USER

```sql
DROP USER 'username'@'host';
DROP USER 'u1'@'h1', 'u2'@'h2';
```

### 5.3 GRANT

```sql
GRANT <privilege> ON <object> TO 'username'@'host';
GRANT SELECT, INSERT ON db.table TO 'alice'@'localhost';
GRANT ALL ON *.* TO 'admin'@'%';
```

### 5.4 REVOKE

```sql
REVOKE <privilege> ON <object> FROM 'username'@'host';
REVOKE SELECT ON db.table FROM 'alice'@'localhost';
```

---

## 6. 系统表设计

### 6.1 mysql.user 列结构

| 列名 | 类型 | 说明 |
|------|------|------|
| `user` | VARCHAR(32) | 用户名 |
| `host` | VARCHAR(255) | 主机 |
| `plugin` | VARCHAR(128) | 认证插件，默认 `caching_sha2_password` |
| `authentication_string` | TEXT | SCRAM encoded verifier |
| `account_locked` | ENUM('Y', 'N') | 账户是否锁定 |

**重要**: 此 schema 预留了 `plugin` 和 `authentication_string` 字段，确保未来兼容 MySQL 8.x 生态工具（如 `SHOW CREATE USER`, `mysqldump`）。

### 6.2 mysql.db 列结构

| 列名 | 类型 | 说明 |
|------|------|------|
| `host` | VARCHAR(255) | 主机 |
| `db` | VARCHAR(64) | 数据库名 |
| `user` | VARCHAR(32) | 用户名 |
| `select_priv` | ENUM('Y', 'N') | SELECT 权限 |
| `insert_priv` | ENUM('Y', 'N') | INSERT 权限 |
| `update_priv` | ENUM('Y', 'N') | UPDATE 权限 |
| `delete_priv` | ENUM('Y', 'N') | DELETE 权限 |
| `create_priv` | ENUM('Y', 'N') | CREATE 权限 |
| `drop_priv` | ENUM('Y', 'N') | DROP 权限 |
| `grant_priv` | ENUM('Y', 'N') | GRANT 权限 |

---

## 7. 权限检查算法

### 7.1 统一入口

```rust
pub fn check_privilege(
    identity: &UserIdentity,
    object: &DbObject,
    privilege: Privilege,
) -> Result<()> {
    // 1. 直接检查用户权限
    if self.has_direct_privilege(identity, privilege, object) {
        return Ok(());
    }

    // 2. 检查 % fallback
    if let Some(wildcard_identity) = self.find_wildcard_user(&identity.username) {
        if self.has_direct_privilege(&wildcard_identity, privilege, object) {
            return Ok(());
        }
    }

    // 3. 拒绝访问
    Err(AuthError::PermissionDenied)
}
```

**重要**: 统一入口函数而非分散的 `check_table_privilege`, `check_db_privilege` 等，便于未来扩展 role inheritance 和 column privilege。

### 7.2 认证流程

```rust
pub fn authenticate(identity: &UserIdentity, password: &str) -> Result<u64> {
    // 1. 查找用户（exact match）
    let user = self.find_exact_user(identity)?;

    // 2. 如果不存在，尝试 % fallback
    let user = user.or_else(|| self.find_wildcard_user(&identity.username))?;

    // 3. 验证密码
    if !user.credential.verify(password) {
        return Err(AuthError::AuthenticationFailed);
    }

    // 4. 检查账户状态
    if !user.is_active {
        return Err(AuthError::AccountLocked);
    }

    Ok(user.id)
}
```

---

## 8. SCRAM 实现细节

### 8.1 凭证生成流程

```rust
fn generate_credential(password: &str) -> ScramCredential {
    let salt = generate_random_salt(32);
    let iterations = ScramCredential::DEFAULT_ITERATIONS;

    let salted_password = pbkdf2(password, &salt, iterations);
    let client_key = hmac_sha256(&salted_password, b"Client Key");
    let stored_key = hash_sha256(&client_key);
    let server_key = hmac_sha256(&salted_password, b"Server Key");

    ScramCredential {
        version: ScramCredential::CURRENT_VERSION,
        salt,
        iterations,
        stored_key,
        server_key,
    }
}
```

### 8.2 密码验证流程

```rust
fn verify(&self, password: &str) -> bool {
    let salted_password = pbkdf2(password, &self.salt, self.iterations);
    let client_key = hmac_sha256(&salted_password, b"Client Key");
    let stored_key = hash_sha256(&client_key);

    constant_time_eq(&stored_key, &self.stored_key)
}
```

---

## 9. 实现计划

| Phase | 内容 | 依赖 |
|-------|------|------|
| 1 | Parser: CREATE USER / DROP USER 语法 | - |
| 2 | AuthStore: UserIdentity + 核心存储 | - |
| 3 | SCRAM: credential 生成/验证 | Phase 2 |
| 4 | Executor: CREATE/DROP USER 执行 | Phase 1, 2, 3 |
| 5 | System Tables: mysql.user, mysql.db 视图 | Phase 2, 4 |
| 6 | Integration: 权限检查执行 | Phase 4, 5 |
| 7 | Tests: 24 场景验收测试 | Phase 1-6 |

---

## 10. 安全边界

### 10.1 必须禁止的行为

| 行为 | 预期结果 |
|------|----------|
| 非管理员执行 CREATE USER | REJECT |
| 非管理员执行 GRANT | REJECT |
| 用户 GRANT 自己没有的权限 | REJECT |
| 用户访问其他用户的数据 | DENY |
| DROP USER 后登录 | REJECT |

### 10.2 默认无权限原则

新创建的用户默认没有任何权限，必须显式 GRANT。

---

## 附录 A: 验收测试矩阵 (24 场景)

### A.1 Authentication (4 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 1 | 密码错误拒绝登录 | reject |
| 2 | 用户不存在拒绝登录 | reject |
| 3 | host 不匹配拒绝登录 | reject |
| 4 | % fallback 正确触发 | accept |

### A.2 Authorization (4 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 5 | 用户 A 无法访问用户 B 数据 | deny |
| 6 | 默认无权限原则成立 | deny all |
| 7 | schema 级权限隔离正确 | db1 ok, db2 deny |
| 8 | 权限继承（未来角色） | 预留 |

### A.3 GRANT/REVOKE Consistency (2 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 9 | GRANT 后可读/可写 | success |
| 10 | GRANT 覆盖行为 | SELECT+INSERT |
| 11 | REVOKE 后权限回收 | success |
| 12 | REVOKE 幂等性 | 两次都 success |

### A.4 User Lifecycle (3 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 13 | CREATE USER 重复创建失败 | ERROR |
| 14 | DROP USER 后认证失效 | reject |
| 15 | DROP USER 不影响其他 host | 其他仍有效 |

### A.5 System Table Projection (3 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 16 | mysql.user 反映 CREATE USER | 可见 |
| 17 | mysql.user 反映 DROP USER | 消失 |
| 18 | mysql.db 反映 GRANT | 可见 |

### A.6 SCRAM Correctness (3 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 19 | salt 随机性 | 不同用户不同 |
| 20 | iteration count 生效 | = 32768 |
| 21 | stored_key ≠ password hash | 确认 |

### A.7 Security Boundary (2 场景)

| # | 场景 | 期望结果 |
|---|------|----------|
| 22 | 无法 GRANT 自己没有的权限 | reject |
| 23 | 非管理员无法 CREATE USER | reject |
| 24 | 非管理员无法 GRANT | reject |

---

## 附录 B: 设计优化记录

| # | 优化项 | 理由 |
|---|--------|------|
| 1 | privileges 使用 HashMap 索引 | O(1) 权限检查 |
| 2 | UserIdentity 规范排序 | 避免 Alice/LOCALHOST 重复 |
| 3 | ScramCredential 加入 version | 未来 SCRAM 升级无需 migration |
| 4 | mysql.user 定义完整列结构 | MySQL 生态工具兼容 |
| 5 | 统一权限检查入口 | 便于扩展 role/column privilege |

---

## 附录 C: 与现有设计的差异

本文档替换 `docs/plans/2026-03-28-rbac-design.md`，主要变更：

1. 添加 UserIdentity 数据结构（支持 user@host）
2. 升级 AuthStore privileges 为 HashMap 索引
3. SCRAM 扩展为完整 credential 结构（含 version）
4. 添加 mysql.user 和 mysql.db 系统表 schema
5. 补充 24 场景验收测试矩阵
6. 调整实现顺序：Parser → AuthStore → SCRAM → Executor → System Tables
