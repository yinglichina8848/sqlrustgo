# Issue #945/#886: Phase 4 安全与治理详细设计

**Issue**: #945 Phase 4: 安全与治理 - RBAC/SSL/审计
**创建日期**: 2026-03-28
**状态**: 设计完成

## 1. 背景与目标

RBAC 权限管理已在 #956 实现并合并。本设计关注：
1. SSL/TLS 加密连接
2. 完整审计日志系统

## 2. SSL/TLS 架构

### 2.1 证书管理

```rust
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_cert_path: Option<PathBuf>,
    pub accept_invalid_certs: bool,
}

pub struct CertificateManager {
    config: TlsConfig,
}

impl CertificateManager {
    pub fn load_certificate(&self) -> Result<Identity>;
    pub fn verify_client(&self, cert: &Certificate) -> Result<()>;
}
```

### 2.2 安全连接

```rust
pub struct SecureConnection {
    stream: TlsStream<TcpStream>,
    peer_addr: SocketAddr,
}

impl SecureConnection {
    pub fn new(stream: TlsStream<TcpStream>) -> Result<Self>;
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    pub fn write(&mut self, buf: &[u8]) -> Result<usize>;
}
```

## 3. 审计日志架构

### 3.1 审计事件类型

```rust
#[derive(Debug, Clone)]
pub enum AuditEvent {
    Login { user: String, success: bool, ip: String },
    Logout { user: String },
    ExecuteSql { user: String, sql: String, duration_ms: u64, rows: u64 },
    DDL { user: String, sql: String },
    DML { user: String, sql: String, table: String },
    Grant { user: String, privilege: String, object: String },
    Revoke { user: String, privilege: String, object: String },
    Error { user: String, error: String },
}
```

### 3.2 审计日志存储

```rust
pub struct AuditLog {
    events: VecDeque<AuditRecord>,
    max_events: usize,
    storage_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub user: String,
    pub ip: String,
    pub details: String,
    pub session_id: u64,
}
```

### 3.3 审计配置

```rust
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: PathBuf,
    pub log_login: bool,
    pub log_sql: bool,
    pub log_ddl: bool,
    pub log_dml: bool,
    pub log_errors: bool,
    pub retention_days: u32,
    pub async_write: bool,
}
```

## 4. 核心组件

### 4.1 AuditManager

```rust
pub struct AuditManager {
    config: AuditConfig,
    session_manager: SessionManager,
}

impl AuditManager {
    pub fn log_event(&self, event: AuditEvent);
    pub fn log_login(&self, user: &str, success: bool, ip: &str);
    pub fn log_sql(&self, user: &str, sql: &str, duration_ms: u64, rows: u64);
    pub fn log_ddl(&self, user: &str, sql: &str);
    pub fn log_grant(&self, user: &str, privilege: &str, object: &str);
    pub fn query_logs(&self, filter: &AuditFilter) -> Vec<AuditRecord>;
}
```

### 4.2 SessionManager

```rust
pub struct SessionManager {
    sessions: HashMap<u64, Session>,
    next_session_id: u64,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: u64,
    pub user: String,
    pub ip: String,
    pub login_time: u64,
    pub last_activity: u64,
    pub is_active: bool,
}

impl SessionManager {
    pub fn create_session(&mut self, user: String, ip: String) -> u64;
    pub fn close_session(&mut self, session_id: u64);
    pub fn get_active_sessions(&self) -> Vec<&Session>;
}
```

## 5. 文件结构

```
crates/
├── security/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── audit.rs        # 审计日志
│   │   ├── session.rs      # 会话管理
│   │   └── tls.rs          # TLS 配置
│   └── Cargo.toml
```

## 6. 实现计划

### Phase 1: 审计系统
1. `audit.rs` - AuditEvent, AuditRecord, AuditManager
2. `session.rs` - SessionManager

### Phase 2: SSL/TLS
1. `tls.rs` - TlsConfig, CertificateManager
2. 集成到 server

## 7. 验收标准

- [ ] SQL 执行审计
- [ ] 登录/登出审计
- [ ] DDL/DML 审计
- [ ] GRANT/REVOKE 审计
- [ ] TLS 连接加密（可选）
