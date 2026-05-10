# DCL 执行链路

> Data Control Language: GRANT, REVOKE, CREATE USER, DROP USER, CREATE ROLE, DROP ROLE

## 1. DCL 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        DCL 执行架构                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  GRANT ──────────► Parser ──► Catalog.check() ──► Auth        │
│                     │              │                         │
│  REVOKE ───────────┘              │                         │
│                     │              ▼                         │
│  CREATE USER ──────┴──► Auth ──► System Table               │
│                     │              │                         │
│  CREATE ROLE ──────┴──► Auth ──► Role Hierarchy            │
│                                                              │
│  权限检查 ────────────────────────────────► Permission Check │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 2. 权限模型

### 2.1 权限层次结构

```
┌─────────────────────────────────────────────────────────────┐
│                    权限层次结构                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  GLOBAL (Server)                                             │
│      │                                                       │
│      ├── CREATE USER                                        │
│      ├── CREATE ROLE                                        │
│      ├── SUPER                                             │
│      ├── PROCESS                                            │
│      └── REPLICATION                                        │
│                                                              │
│  DATABASE                                                   │
│      ├── CREATE                                             │
│      ├── DROP                                               │
│      └── GRANT OPTION                                       │
│                                                              │
│  TABLE                                                      │
│      ├── SELECT                                             │
│      ├── INSERT                                             │
│      ├── UPDATE                                             │
│      ├── DELETE                                             │
│      ├── CREATE                                             │
│      ├── DROP                                               │
│      ├── INDEX                                              │
│      └── REFERENCES                                         │
│                                                              │
│  COLUMN                                                     │
│      ├── SELECT                                             │
│      ├── INSERT                                             │
│      └── UPDATE                                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 权限类型枚举

```rust
pub enum Privilege {
    Read,       // SELECT
    Insert,     // INSERT
    Update,     // UPDATE
    Delete,     // DELETE
    Alter,      // ALTER TABLE
    Drop,       // DROP
    Create,     // CREATE TABLE/INDEX
    Grant,      // GRANT OPTION
    Index,      // CREATE INDEX
    References, // REFERENCES
    All,        // ALL PRIVILEGES
}
```

## 3. GRANT 执行链路

### 3.1 GRANT 时序图

```
GRANT SELECT, INSERT ON database.table TO 'user'@'host'
    │
    ▼
┌─────────────────────────────────────────────┐
│              Parser                           │
│  GRANT Statement {                            │
│    privileges: [SELECT, INSERT],             │
│    object_type: TABLE,                       │
│    object_name: "database.table",            │
│    grantee: "user@host"                     │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Privilege Checker               │
│  ┌─────────────────────────────────────┐   │
│  │ 检查 GRANTOR 是否拥有 GRANT OPTION   │   │
│  │ 检查权限是否可授予该对象类型         │   │
│  │ 检查目标用户是否存在                 │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Catalog                        │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 解析对象名 (db.table)            │   │
│  │ 2. 查找或创建用户                    │   │
│  │ 3. 写入 mysql.db 系统表             │   │
│  │ 4. 写入 mysql.tables_priv 表       │   │
│  │ 5. 更新内存缓存                      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Auth Manager                    │
│  grant_privilege(                           │
│    identity: UserIdentity,                  │
│    privilege: Privilege,                   │
│    object_type: ObjectType,                │
│    object_name: &str                       │
│  )                                         │
└─────────────────────────────────────────────┘
```

### 3.2 GRANT 状态图

```
                    ┌──────────────────┐
                    │    INITIAL       │
                    └────────┬─────────┘
                             │ parse GRANT
                             ▼
                    ┌──────────────────┐
         ┌─────────►│   PARSED         │
         │          └────────┬─────────┘
         │                   │ validate syntax
         │                   ▼
         │          ┌──────────────────┐
         │          │  CHECK_PRIVS     │
         │          └────────┬─────────┘
         │                   │ GRANTOR has privilege?
         │          ┌────────┴─────────┐
         │          │ NO               │ YES
         │          ▼                  ▼
         │   ┌────────────┐    ┌──────────────────┐
         │   │  ERROR:    │    │   CHECK_USER     │
         │   │  Access    │    └────────┬─────────┘
         │   │  Denied    │             │ User exists?
         │   └────────────┘    ┌────────┴─────────┐
         │                     │ NO               │ YES
         │                     ▼                  ▼
         │            ┌─────────────┐    ┌──────────────────┐
         │            │   CREATE    │    │   CHECK_OBJECT   │
         │            │   USER      │    └────────┬─────────┘
         │            └─────────────┘             │ Object exists?
         │                              ┌─────────┴─────────┐
         │                              │ NO               │ YES
         │                              ▼                  ▼
         │                      ┌─────────────┐    ┌──────────────────┐
         │                      │   ERROR:    │    │   WRITE_GRANT    │
         │                      │  No Such   │    └────────┬─────────┘
         │                      │  Object    │             │
         │                      └─────────────┘             ▼
         │                                     ┌──────────────────┐
         │                                     │   COMMITTED      │
         │                                     └──────────────────┘
         │
         │                   ┌──────────────────┐
         └───────────────────│     ERROR        │
                             └──────────────────┘
```

## 4. REVOKE 执行链路

### 4.1 REVOKE 时序图

```
REVOKE INSERT ON database.table FROM 'user'@'host'
    │
    ▼
┌─────────────────────────────────────────────┐
│              Parser                           │
│  REVOKE Statement {                          │
│    privileges: [INSERT],                     │
│    object_type: TABLE,                       │
│    object_name: "database.table",            │
│    grantee: "user@host"                     │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Privilege Checker               │
│  ┌─────────────────────────────────────┐   │
│  │ 检查 REVOKER 是否拥有 GRANT OPTION   │   │
│  │ 检查权限是否存在授予记录              │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Catalog                        │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 查找用户权限记录                  │   │
│  │ 2. 删除对应权限记录                 │   │
│  │ 3. 更新内存缓存                      │   │
│  │ 4. 记录审计日志                      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Auth Manager                    │
│  revoke_privilege(                          │
│    identity: UserIdentity,                  │
│    privilege: Privilege,                   │
│    object_type: ObjectType,                │
│    object_name: &str                        │
│  )                                         │
└─────────────────────────────────────────────┘
```

## 5. 角色管理

### 5.1 角色层次结构

```rust
/// Role with inheritance support
pub struct Role {
    pub name: String,
    pub privileges: Vec<Privilege>,
    pub parent_roles: Vec<String>,  // Role inheritance
    pub is_default: bool,
}
```

### 5.2 角色授予时序图

```
GRANT 'role_name' TO 'user'@'host'
    │
    ▼
┌─────────────────────────────────────────────┐
│              Parser                         │
│  GRANT Role Statement {                      │
│    role: "role_name",                       │
│    grantee: "user@host"                     │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Auth Manager                    │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 检查角色是否存在                   │   │
│  │ 2. 检查用户是否存在                   │   │
│  │ 3. 检查是否已授予该角色              │   │
│  │ 4. 写入 mysql.role_edges 表         │   │
│  │ 5. 更新用户有效权限缓存              │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## 6. 权限检查流程

### 6.1 权限检查时序图

```
SELECT * FROM orders WHERE customer_id = 1
    │
    ▼
┌─────────────────────────────────────────────┐
│              Session                        │
│  current_user: "app@192.168.1.%"           │
│  session_context: {                         │
│    current_database: "ecommerce"          │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Auth Manager                    │
│  check_permission(                           │
│    user: "app@192.168.1.%",               │
│    privilege: SELECT,                       │
│    object: ("ecommerce", "orders")         │
│  )                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Privilege Resolution            │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 检查全局权限 (mysql.user)        │   │
│  │ 2. 检查数据库权限 (mysql.db)        │   │
│  │ 3. 检查表级权限 (mysql.tables_priv)│   │
│  │ 4. 检查列级权限 (mysql.columns_priv)│   │
│  │ 5. 合并角色权限                     │   │
│  │ 6. 返回最终权限向量                 │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 6.2 权限检查状态机

```
                  ┌──────────────────┐
                  │    INITIAL       │
                  └────────┬─────────┘
                           │ need privilege P on object O
                           ▼
                  ┌──────────────────┐
                  │  CHECK_GLOBAL    │
                  └────────┬─────────┘
                           │ GLOBAL grant exists?
                  ┌────────┴─────────┐
                  │ YES              │ NO
                  ▼                  ▼
          ┌──────────────┐    ┌──────────────────┐
          │   ALLOWED    │    │  CHECK_DATABASE  │
          └──────────────┘    └────────┬─────────┘
                                        │ DB grant exists?
                              ┌─────────┴─────────┐
                              │ YES               │ NO
                              ▼                   ▼
                      ┌──────────────┐    ┌──────────────────┐
                      │   ALLOWED    │    │   CHECK_TABLE    │
                      └──────────────┘    └────────┬─────────┘
                                                    │ TABLE grant exists?
                                        ┌──────────┴──────────┐
                                        │ YES                │ NO
                                        ▼                    ▼
                                ┌──────────────┐    ┌──────────────────┐
                                │   ALLOWED    │    │  CHECK_COLUMN    │
                                └──────────────┘    └────────┬─────────┘
                                                            │ COLUMN grant exists?
                                              ┌─────────────┴─────────────┐
                                              │ YES                       │ NO
                                              ▼                           ▼
                                      ┌──────────────┐          ┌──────────────┐
                                      │   ALLOWED    │          │    DENIED    │
                                      └──────────────┘          └──────────────┘
```

## 7. 用户认证

### 7.1 SCRAM-SHA-256 认证流程

```
Client                                          Server
  │                                               │
  │  ──── Connection Request ──────────────────► │
  │                                               │
  │  ◄──── Challenge (server_nonce) ───────────── │
  │                                               │
  │  ──── Auth Response (client_proof) ─────────► │
  │                                               │
  │              ┌─────────────────────────┐      │
  │              │ 1. 验证 server_nonce     │      │
  │              │ 2. 计算 client_key       │      │
  │              │ 3. 计算 stored_key      │      │
  │              │ 4. 验证 client_proof    │      │
  │              │ 5. 计算 server_proof    │      │
  │              └─────────────────────────┘      │
  │                                               │
  │  ◄──── Auth Success ──────────────────────── │
```

### 7.2 密码验证算法

```rust
impl ScramCredential {
    pub fn verify(&self, password: &str) -> bool {
        // PBKDF2 with salt and iterations
        let salted_password = pbkdf2(password, &self.salt, self.iterations);

        // Derive client_key
        let client_key = hmac_sha256(&salted_password, b"Client Key");

        // Hash to get stored_key
        let stored_key = hash_sha256(&client_key);

        // Constant-time comparison
        constant_time_eq(&stored_key, &self.stored_key)
    }
}
```

## 8. 系统表结构

### 8.1 mysql.user 表

| 字段 | 类型 | 说明 |
|------|------|------|
| host | CHAR(255) | 主机 |
| user | CHAR(32) | 用户名 |
| select_priv | ENUM | SELECT 权限 |
| insert_priv | ENUM | INSERT 权限 |
| update_priv | ENUM | UPDATE 权限 |
| delete_priv | ENUM | DELETE 权限 |
| create_priv | ENUM | CREATE 权限 |
| drop_priv | ENUM | DROP 权限 |
| grant_priv | ENUM | GRANT 权限 |
| ssl_type | ENUM | SSL 类型 |

### 8.2 mysql.db 表

| 字段 | 类型 | 说明 |
|------|------|------|
| host | CHAR(255) | 主机 |
| db | CHAR(64) | 数据库名 |
| user | CHAR(32) | 用户名 |
| select_priv | ENUM | SELECT 权限 |
| insert_priv | ENUM | INSERT 权限 |
| ... | ... | 其他权限 |

### 8.3 mysql.tables_priv 表

| 字段 | 类型 | 说明 |
|------|------|------|
| host | CHAR(255) | 主机 |
| db | CHAR(64) | 数据库名 |
| user | CHAR(32) | 用户名 |
| table_name | CHAR(64) | 表名 |
| grantor | CHAR(93) | 授权者 |
| timestamp | TIMESTAMP | 时间戳 |
| table_priv | SET | 表级权限 |
| column_priv | SET | 列级权限 |

## 9. 审计日志

### 9.1 审计事件类型

```rust
pub enum AuditEvent {
    // User management
    CreateUser { user: String, host: String },
    DropUser { user: String, host: String },
    AlterUser { user: String, host: String },

    // Privilege management
    Grant {
        grantee: String,
        privilege: Privilege,
        object: String,
    },
    Revoke {
        grantee: String,
        privilege: Privilege,
        object: String,
    },

    // Role management
    CreateRole { role: String },
    DropRole { role: String },
    GrantRole { role: String, user: String },
    RevokeRole { role: String, user: String },

    // Authentication
    LoginSuccess { user: String, host: String },
    LoginFailure { user: String, host: String },

    // Permission denied
    AccessDenied {
        user: String,
        privilege: Privilege,
        object: String,
    },
}
```

## 10. 测试计划

### 10.1 测试用例

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| DCL-T01 | GRANT SELECT ON table TO user | 权限授予成功 |
| DCL-T02 | REVOKE SELECT ON table FROM user | 权限撤销成功 |
| DCL-T03 | GRANT with non-existent user | 自动创建用户 |
| DCL-T04 | REVOKE non-existent privilege | 返回错误 |
| DCL-T05 | GRANT without GRANT OPTION | 返回错误 |
| DCL-T06 | Role creation and assignment | 角色授予成功 |
| DCL-T07 | Role inheritance chain | 权限继承正确 |
| DCL-T08 | Column-level privileges | 列级权限检查 |
| DCL-T09 | Concurrent GRANT/REVOKE | 无竞态条件 |
| DCL-T10 | Audit log completeness | 所有事件记录 |

### 10.2 安全测试

| 测试编号 | 测试内容 | 攻击类型 |
|----------|----------|----------|
| DCL-S01 | SQL 注入用户/密码 | 注入攻击 |
| DCL-S02 | 权限提升尝试 | 提权攻击 |
| DCL-S03 | 跨数据库访问 | 横向移动 |
| DCL-S04 | 暴力破解密码 | 暴力攻击 |
| DCL-S05 | 时间盲注验证 | 侧信道攻击 |

## 11. 覆盖率差距分析

### 11.1 当前覆盖率

| 组件 | 行覆盖率 | 差距 |
|------|----------|------|
| auth.rs | ~65% | UserIdentity, ScramCredential, Privilege |
| catalog.rs (grant/revoke) | ~40% | DCL executor 不完整 |
| system_tables/mysql_db.rs | ~30% | mysql.db 表查询 |
| system_tables/tables_priv.rs | ~25% | 表级权限查询 |
| system_tables/columns_priv.rs | ~20% | 列级权限查询 |

### 11.2 差距原因

1. **DCL Executor 缺失**: 没有独立的 DCL 执行器，DML/DDL 执行器未处理 GRANT/REVOKE
2. **系统表不完整**: 缺少 mysql.role_edges, mysql.proxies_priv 等表
3. **审计日志不完整**: 只记录了登录事件，未记录权限变更
4. **密码策略缺失**: 无密码过期、密码历史、密码复杂度检查

### 11.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 实现 DCL Executor | 70% |
| v3.1.0 | 补充系统表 | 75% |
| v3.2.0 | 完整审计日志 | 85% |
| v3.2.0 | 密码策略 | 80% |

## 12. 核心文件索引

| 文件 | 说明 |
|------|------|
| `crates/catalog/src/auth.rs` | RBAC 实现，1900+ 行 |
| `crates/catalog/src/catalog.rs` | Catalog grant/revoke 方法 |
| `crates/catalog/src/system_tables/mysql_db.rs` | mysql.db 系统表 |
| `crates/catalog/src/system_tables/tables_priv.rs` | 表级权限系统表 |
| `crates/security/src/session.rs` | 会话权限上下文 |
| `crates/security/src/audit.rs` | 审计日志 |
| `crates/server/src/security_integration.rs` | 安全集成 |

## 13. 相关文档

| 文档 | 说明 |
|------|------|
| [DDL_EXECUTION.md](../ddl/DDL_EXECUTION.md) | DDL 执行链路 |
| [DML_EXECUTION.md](../dml/DML_EXECUTION.md) | DML 执行链路 |
| [SECURITY_REPORT.md](../../SECURITY_REPORT.md) | 安全报告 |
