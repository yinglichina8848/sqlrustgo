# 配置加固指南

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 一、审计系统强制配置

### 1.1 签名验证必须开启

```yaml
# /etc/sqlrustgo/sqlrustgo.toml
[audit]
# 强制开启签名验证 - 无法禁用
signature_verification = "enforce"  # enforce, warn, disabled

# Hash Chain 必须开启
hash_chain_enabled = true

# 审计日志不可关闭
audit_trail = "mandatory"  # mandatory, optional, disabled
```

### 1.2 签名密钥配置

```yaml
[audit.signature]
# 密钥路径 (建议使用 HSM)
key_path = "/etc/sqlrustgo/keys/audit_private.pem"

# 密钥轮换周期 (天)
rotation_period_days = 90

# 自动轮换
auto_rotate = true
```

---

## 二、网络安全配置

### 2.1 TLS 配置

```yaml
[network]
# 强制 TLS 1.2+
tls_enabled = true
tls_min_version = "1.2"

# 证书配置
cert_path = "/etc/sqlrustgo/tls/server.crt"
key_path = "/etc/sqlrustgo/tls/server.key"

# 客户端认证
client_auth = "require"  # require, request, none
ca_path = "/etc/sqlrustgo/tls/ca.crt"
```

### 2.2 连接安全

```yaml
[network.connection]
# 最大连接数
max_connections = 1000

# 连接超时 (秒)
connect_timeout = 30

# 空闲超时 (秒)
idle_timeout = 3600

# 心跳间隔 (秒)
heartbeat_interval = 60
```

---

## 三、访问控制配置

### 3.1 认证配置

```yaml
[auth]
# 认证方式
method = "password"  # password, ldap, oauth, radius

# 密码策略
password_policy {
  min_length = 12
  require_uppercase = true
  require_lowercase = true
  require_digit = true
  require_special = true
  max_age_days = 90
  history_count = 5
}

# 登录失败锁定
failed_login_lockout {
  max_attempts = 5
  lockout_duration_minutes = 30
}
```

### 3.2 授权配置

```yaml
[authz]
# 默认角色
default_role = "read_only"

# 角色定义
roles {
  admin = ["ALL PRIVILEGES"]
  audit_viewer = ["SELECT ON audit_log", "EXECUTE ON audit_verify"]
  application = [
    "SELECT, INSERT, UPDATE, DELETE ON business_tables",
    "SELECT ON reference_data"
  ]
  readonly = ["SELECT ON ALL"]
}
```

---

## 四、存储安全配置

### 4.1 数据加密 (v3.2.0)

```yaml
[storage.encryption]
# 页级加密
enabled = true
algorithm = "AES-256-GCM"

# 密钥管理
key_manager = "kms"  # kms, file, hsm
kms_endpoint = "https://kms.internal:8080"
```

### 4.2 WAL 安全

```yaml
[storage.wal]
# WAL 加密
encrypt = true

# WAL 完整性校验
checksum = "CRC32"

# Checkpoint 前强制刷盘
force_flush_before_checkpoint = true
```

---

## 五、审计配置

### 5.1 审计事件类型

```yaml
[audit.events]
# 必须审计的事件类型
required = [
  "SESSION_START",
  "SESSION_END",
  "LOGIN_SUCCESS",
  "LOGIN_FAILURE",
  "QUERY_EXECUTE",
  "TRANSACTION_BEGIN",
  "TRANSACTION_COMMIT",
  "TRANSACTION_ROLLBACK",
  "DDL_EXECUTE",
  "USER_CREATE",
  "USER_MODIFY",
  "USER_DELETE",
  "GRANT",
  "REVOKE"
]

# 警告事件 (可选审计)
warn = [
  "LARGE_QUERY",
  "SLOW_QUERY",
  "LOCK_TIMEOUT"
]
```

### 5.2 审计存储

```yaml
[audit.storage]
# 独立存储卷
separate_volume = true
volume_path = "/var/lib/sqlrustgo/audit"

# 追加模式 (不可修改)
append_only = true

# 本地保留期 (天)
retention_days = 2555  # 7 years for GMP

# 备份配置
backup {
  enabled = true
  interval_hours = 24
  destination = "s3://audit-backup-prod/"
  encryption = true
}
```

---

## 六、监控与告警

### 6.1 审计健康监控

```yaml
[monitoring.audit]
# 审计链断裂告警
hash_chain_broken_alert = true

# 签名验证失败告警
signature_failure_alert = true

# 审计存储满告警
storage_threshold_percent = 90

# 审计写入失败告警
write_failure_alert = true
```

### 6.2 告警配置

```yaml
[monitoring.alerts]
email = ["ops@company.com", "security@company.com"]
slack_webhook = "https://hooks.slack.com/..."
pagerduty_key = "..."

# 告警规则
rules {
  audit_chain_broken = "critical"
  signature_failure = "critical"
  audit_storage_full = "high"
  failed_login = "medium"
}
```

---

## 七、合规检查清单

### 7.1 GMP 合规配置

| 检查项 | 配置项 | 验证命令 |
|--------|--------|----------|
| ✅ 审计启用 | `audit.trail = mandatory` | `SHOW VARIABLES LIKE 'audit_trail'` |
| ✅ 签名验证 | `audit.signature = enforce` | `SHOW VARIABLES LIKE 'audit_signature'` |
| ✅ Hash Chain | `audit.hash_chain = true` | `SELECT COUNT(*) FROM audit_hash_chain` |
| ✅ TLS 1.2+ | `network.tls_min_version = 1.2` | `SHOW VARIABLES LIKE 'tls_version'` |
| ✅ 密码策略 | `auth.password_policy.*` | `SHOW VARIABLES LIKE 'password_policy'` |
| ✅ 审计保留 | `audit.retention_days = 2555` | `SHOW VARIABLES LIKE 'audit_retention'` |
| ⚠️ 存储加密 | `storage.encryption = true` | v3.2.0 |

### 7.2 安全基线

```bash
# 验证审计系统健康
./sqlrustgo audit health-check

# 验证 Hash Chain 完整性
./sqlrustgo audit verify --chain

# 验证签名
./sqlrustgo audit verify --signatures

# 生成合规报告
./sqlrustgo audit compliance-report --format pdf
```