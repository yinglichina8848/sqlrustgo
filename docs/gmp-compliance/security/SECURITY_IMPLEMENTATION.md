# 安全控制实现分析

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 1. 认证机制

### 1.1 当前实现

SQLRustGo v3.1.0 使用基于密码的简单认证:

```rust
// crates/security/src/auth.rs

/// 认证请求
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// 认证响应
pub enum AuthResponse {
    Success { session_id: String },
    Failure { reason: String },
}

/// 认证管理器
pub struct AuthManager {
    users: HashMap<String, User>,
}

impl AuthManager {
    /// 验证凭证
    pub fn authenticate(&self, req: &AuthRequest) -> AuthResponse {
        match self.users.get(&req.username) {
            Some(user) => {
                if self.verify_password(&req.password, &user.password_hash) {
                    AuthResponse::Success {
                        session_id: self.create_session(user.id),
                    }
                } else {
                    AuthResponse::Failure {
                        reason: "Invalid password".to_string(),
                    }
                }
            }
            None => AuthResponse::Failure {
                reason: "User not found".to_string(),
            },
        }
    }
    
    /// 密码验证 (使用 bcrypt)
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }
}
```

### 1.2 密码存储

| 配置 | 值 | 说明 |
|------|-----|------|
| 算法 | bcrypt | 工作因子 12 |
| 哈希格式 | `$2b$12$...` | 标准 bcrypt 格式 |
| 盐 | 自动生成 | 128 位 |

### 1.3 认证流程

```
Client                          Server
   |                               |
   |--- AUTH: user/pass --------->|
   |                               | 1. 查询用户
   |                               | 2. 验证 bcrypt 哈希
   |                               | 3. 生成 session
   |<-- AUTH_OK: session_id ------|
   |                               |
   |--- QUERY: session_id ------->|
   |                               | 4. 验证 session
   |<-- RESULT -------------------|
```

### 1.4 当前限制

| 限制 | 说明 | 缓解 |
|------|------|------|
| 无 SCRAM | 当前使用简单密码传输 | 建议 TLS |
| 无 LDAP | 不支持 LDAP 集成 | v3.2.0 |
| 无 OAuth | 不支持 OAuth | v3.2.0 |
| 无 MFA | 无多因素认证 | v3.2.0 |

### 1.5 建议的安全配置

```yaml
# 生产环境配置
auth:
  # 强制 TLS
  require_tls: true
  
  # 密码策略
  password_policy:
    min_length: 12
    bcrypt_cost: 12
    
  # 登录限制
  max_login_attempts: 5
  lockout_duration_minutes: 30
```

---

## 2. 授权机制

### 2.1 当前实现

SQLRustGo v3.1.0 使用简单的 RBAC (基于角色的访问控制):

```rust
// crates/security/src/authz.rs

/// 角色定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,          // 完全权限
    ReadWrite,      // SELECT, INSERT, UPDATE, DELETE
    ReadOnly,       // SELECT
    AuditViewer,    // SELECT ON audit_log
}

/// 权限
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Privilege {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Execute,
}

/// 检查权限
pub fn check_privilege(role: &Role, privilege: &Privilege) -> bool {
    match role {
        Role::Admin => true,
        Role::ReadWrite => matches!(privilege, 
            Privilege::Select | 
            Privilege::Insert | 
            Privilege::Update | 
            Privilege::Delete
        ),
        Role::ReadOnly => matches!(privilege, Privilege::Select),
        Role::AuditViewer => matches!(privilege, Privilege::Select),
    }
}
```

### 2.2 权限矩阵

| 角色 | SELECT | INSERT | UPDATE | DELETE | CREATE/DROP | AUDIT |
|------|--------|--------|--------|--------|-------------|-------|
| Admin | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ReadWrite | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| ReadOnly | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| AuditViewer | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |

### 2.3 权限检查流程

```rust
impl Session {
    /// 执行查询前检查权限
    pub fn check_access(&self, query: &Query) -> Result<(), AccessDenied> {
        match query {
            Query::Select { table, .. } => {
                if !check_privilege(&self.role, &Privilege::Select) {
                    return Err(AccessDenied::NoSelectPrivilege(table.clone()));
                }
            }
            Query::Insert { table, .. } => {
                if !check_privilege(&self.role, &Privilege::Insert) {
                    return Err(AccessDenied::NoInsertPrivilege(table.clone()));
                }
            }
            // ...
        }
        Ok(())
    }
}
```

### 2.4 当前限制

| 限制 | 说明 | GMP 影响 |
|------|------|----------|
| 无列级权限 | 权限只能到表 | 低 (应用层控制) |
| 无行级权限 | 无 RLS | 中 (需要应用层过滤) |
| 无资源配额 | 无 CPU/内存限制 | 低 (操作系统级) |
| 无审计权限 | 管理员可修改审计 | 高 (需分离职责) |

### 2.5 建议的安全配置

```yaml
# 生产环境 - 职责分离
authz:
  # 管理员不能直接操作审计表
  separation_of_duties:
    enabled: true
    admin_cannot_access:
      - audit_log
      - system_audit
      
  # 审计管理员角色
  audit_admin:
    privileges:
      - SELECT ON audit_log
      - SELECT ON audit_hash_chain
      - EXECUTE ON audit_verify
    cannot:
      - DELETE ON audit_log
      - UPDATE ON audit_log
```

---

## 3. 传输加密

### 3.1 TLS 配置

```rust
// crates/network/src/tls.rs

pub struct TlsConfig {
    pub enabled: bool,
    pub min_version: TlsVersion,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub client_auth: ClientAuth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
}

impl TlsConfig {
    pub fn validate(&self) -> Result<(), TlsError> {
        // 强制 TLS 1.2+
        if self.min_version < TlsVersion::V1_2 {
            return Err(TlsError::InsecureVersion);
        }
        
        // 验证证书
        if !self.cert_path.exists() {
            return Err(TlsError::CertNotFound);
        }
        
        Ok(())
    }
}
```

### 3.2 TLS 配置示例

```yaml
# 生产环境 TLS 配置
network:
  tls:
    enabled: true
    min_version: "1.2"          # 强制 TLS 1.2+
    cert_path: "/etc/sqlrustgo/tls/server.crt"
    key_path: "/etc/sqlrustgo/tls/server.key"
    client_auth: "require"        # 强制客户端证书
    
  # 密码套件
  cipher_suites:
    - "TLS_AES_256_GCM_SHA384"
    - "TLS_AES_128_GCM_SHA256"
    - "TLS_CHACHA20_POLY1305_SHA256"
```

### 3.3 当前状态

| 配置 | 状态 | 说明 |
|------|------|------|
| TLS 支持 | ✅ | 基础实现 |
| TLS 1.2+ | ✅ | 强制 |
| TLS 1.3 | ⚠️ | 依赖系统库 |
| 客户端证书 | ⚠️ | 可选配置 |
| 证书固定 | ❌ | v3.2.0 |

---

## 4. 审计存储防篡改

### 4.1 多层防护

```
┌─────────────────────────────────────────────────────────────┐
│                    防护层 (Defense in Depth)                  │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Application (审计系统)                              │
│   - Hash Chain (SHA-256)                                    │
│   - Digital Signature (Ed25519)                             │
│   - Event Stream (Append-only)                              │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Storage (WAL)                                      │
│   - WAL Write-Ahead Logging                                 │
│   - CRC32 Checksum                                          │
│   - Checkpoint                                              │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: OS (文件系统)                                       │
│   - Append-only (chattr +a)                                │
│   - SELinux/AppArmor                                        │
│   - 独立磁盘卷                                               │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Infrastructure (硬件)                               │
│   - RAID 电池备份                                           │
│   - UPS                                                     │
│   - 异地复制                                                │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 OS 级保护

```bash
# 1. 设置 append-only 权限
chattr +a /var/lib/sqlrustgo/audit/*.log

# 2. 验证
lsattr /var/lib/sqlrustgo/audit/*.log
# -----a-------------- /var/lib/sqlrustgo/audit/audit.log

# 3. 设置文件权限
chown sqlrustgo:sqlrustgo /var/lib/sqlrustgo/audit
chmod 700 /var/lib/sqlrustgo/audit

# 4. SELinux 策略 (示例)
# allow sqlrustgo_t audit_log_t:file { append create open };
# deny sqlrustgo_t audit_log_t:file { unlink rename write };
```

### 4.3 防护效果

| 攻击类型 | Layer 1 | Layer 2 | Layer 3 | Layer 4 |
|----------|---------|---------|---------|---------|
| 篡改审计内容 | ✅ 哈希链 | ✅ WAL | ✅ Append-only | ✅ 异地备份 |
| 删除审计事件 | ✅ 哈希链 | ✅ WAL | ✅ Append-only | ✅ 异地备份 |
| 伪造审计事件 | ✅ 签名 | ✅ WAL | ✅ | ✅ |
| 物理磁盘损坏 | ✅ | ✅ Checkpoint | ✅ RAID | ✅ 异地 |
| 内部篡改 | ⚠️ 分离职责 | ✅ | ✅ 权限控制 | ✅ 异地 |

---

## 5. GMP 关键安全控制矩阵

| GMP 控制项 | 实现状态 | 证据 | 优先级 |
|------------|----------|------|--------|
| 认证 | ✅ | AuthManager + bcrypt | P0 |
| 授权 | ✅ | RBAC + check_privilege | P0 |
| 审计日志 | ✅ | HashChain + Signature | P0 |
| TLS 传输 | ⚠️ | TlsConfig | P1 |
| 密码策略 | ✅ | bcrypt cost 12 | P1 |
| 会话管理 | ✅ | Session + timeout | P1 |
| 完整性校验 | ✅ | CRC32 + SHA-256 | P0 |
| 备份 | ⚠️ | 需配置 | P1 |

---

## 6. 剩余安全缺口与缓解

| 缺口 | 风险 | 缓解措施 | 计划版本 |
|------|------|----------|----------|
| 无 LDAP 集成 | 无法与企业 SSO 集成 | 使用 TLS + 本地用户 | v3.2.0 |
| 无 MFA | 账号被盗风险 | TLS + 强密码 | v3.2.0 |
| 无列级权限 | 精细控制不足 | 应用层过滤 | v3.2.0 |
| 无行级安全 | 数据隔离不足 | 应用层过滤 | v3.2.0 |
| 无独立审计管理员 | DBA 可篡改 | 职责分离配置 | v3.1.0 |
| 无加密存储 | 数据泄露风险 | 全盘加密 | v3.2.0 |

### 6.1 职责分离配置

```yaml
# 启用 DBA 和审计管理员分离
authz:
  separation_of_duties:
    enabled: true
    
  # DBA 角色 (不能访问审计)
  dba:
    privileges:
      - ALL ON *
    cannot:
      - SELECT ON audit_log
      - SELECT ON system_audit
      
  # 审计管理员 (不能修改数据)
  audit_admin:
    privileges:
      - SELECT ON audit_log
      - EXECUTE ON audit_verify
    cannot:
      - INSERT ON *
      - UPDATE ON *
      - DELETE ON *
```