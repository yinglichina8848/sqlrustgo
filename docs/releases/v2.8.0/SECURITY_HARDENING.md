# SQLRustGo Security Hardening Guide

> Version: `v2.8.0`
> Last Updated: 2026-04-23

---

## Overview

This guide covers security hardening for SQLRustGo production deployments.

## Authentication

### MySQL Authentication

SQLRustGo supports MySQL Old Password authentication:

```bash
# Connect with mysql client
mysql -h 127.0.0.1 -P 3306 -u root
```

### Password Hashing

SQLRustGo uses MySQL 4.x/5.x compatible password hashing:

```rust
fn old_password_hash(password: &str) -> [u8; 8] {
    // MySQL OLD_PASSWORD algorithm
}
```

## Authorization

### Column-Level Permissions (T-17)

SQLRustGo supports column-level access control:

```rust
// ColumnMasker example
let masker = ColumnMasker::new_with_config(MaskingConfig {
    rules: vec![
        MaskingRule {
            id: "mask_email".to_string(),
            column: "email".to_string(),
            mask_type: MaskingType::Partial,
            description: "Mask email addresses".to_string(),
        },
    ],
});
```

### GRANT/REVOKE

Column-level GRANT/REVOKE support:

```sql
-- Grant column-level permission
GRANT SELECT(email) ON users TO 'reader'@'localhost';
REVOKE SELECT(salary) ON employees FROM 'app'@'localhost';
```

---

## Audit Logging

### Audit System (T-18)

SQLRustGo provides comprehensive audit logging:

```rust
use sqlrustgo_security::{AuditManager, AuditEvent};

let audit = AuditManager::new();
audit.log(AuditEvent::ExecuteSql {
    user: "root".to_string(),
    sql: "SELECT * FROM users".to_string(),
    duration_ms: 5,
    rows: 100,
    session_id: 12345,
});
```

### Audit Events

| Event | Description |
|-------|-------------|
| `Login` | User login attempt |
| `Logout` | User logout |
| `ExecuteSql` | SQL execution |
| `DDL` | CREATE/ALTER/DROP statements |
| `DML` | INSERT/UPDATE/DELETE |
| `Grant` | Permission grant |
| `Revoke` | Permission revoke |
| `Error` | Error occurrences |

### Audit Filter

Filter audit records by user or event type:

```rust
let filter = AuditFilter::new()
    .with_users(vec!["admin".to_string()])
    .with_event_types(vec!["DDL".to_string(), "DML".to_string()]);

let records = audit.query(&filter);
```

---

## SQL Firewall

### Injection Prevention

SQL Firewall blocks malicious queries:

```rust
use sqlrustgo_security::{SqlFirewall, FirewallConfig, ThreatSeverity};

let firewall = SqlFirewall::new(FirewallConfig {
    block_injection: true,
    block_ddl: false,
    threat_severity_threshold: ThreatSeverity::Medium,
});
```

### Blacklist Patterns

```rust
firewall.add_blacklist_pattern(
    BlacklistPattern::new("DROP TABLE".to_string())
);
```

### Whitelist Patterns

```rust
firewall.add_whitelist_pattern(
    WhitelistPattern::new("SELECT * FROM audit_logs".to_string())
);
```

---

## TLS/SSL Configuration

### Certificate Management

```rust
use sqlrustgo_security::{CertificateManager, TlsConfig};

let cert_manager = CertificateManager::new();
let tls_config = TlsConfig::builder()
    .with_certificate("/path/to/cert.pem")
    .with_private_key("/path/to/key.pem")
    .with_min_tls_version("1.2")
    .build();
```

### Generate Self-Signed Certificate

```bash
# Generate self-signed certificate for testing
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

---

## Session Management

### Session Security

```rust
use sqlrustgo_security::{SessionManager, SessionStatus};

let session_manager = SessionManager::new();
let session = session_manager.create_session("root", "127.0.0.1");

// Set session timeout
session.set_timeout(Duration::from_secs(3600));

// Require SSL for session
session.require_ssl();
```

---

## Data Encryption

### TDE (Transparent Data Encryption)

SQLRustGo supports AES-256 encryption for data at rest:

```rust
// Enable transparent encryption
storage.enable_encryption(Aes256Config {
    key_path: "/secure/encryption.key",
});
```

### Column Encryption

Encrypt sensitive columns:

```sql
-- Create table with encrypted column
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    ssn TEXT ENCRYPTED,
    credit_card TEXT ENCRYPTED
);
```

---

## Security Checklist

### Pre-Deployment

- [ ] Change default root password
- [ ] Enable SSL/TLS
- [ ] Configure firewall rules
- [ ] Enable audit logging
- [ ] Set session timeouts
- [ ] Configure SQL injection protection

### Production Checklist

- [ ] Use strong passwords (min 12 chars, mix case/number/symbol)
- [ ] Enable audit logging for all DDL/DML
- [ ] Regular security updates
- [ ] Monitor failed login attempts
- [ ] Backup audit logs regularly
- [ ] Test recovery procedures

---

## Vulnerability Mitigation

### SQL Injection

**Risk**: User input interpreted as SQL code

**Mitigation**:
- Use parameterized queries
- Enable SQL Firewall
- Validate and sanitize all input

### Authentication Bypass

**Risk**: Weak authentication mechanism

**Mitigation**:
- Use strong password policies
- Enable multi-factor authentication
- Implement account lockout policies

### Data Exposure

**Risk**: Sensitive data in logs or error messages

**Mitigation**:
- Enable column-level encryption
- Mask sensitive data in logs
- Configure error handling to hide details

---

## Compliance

### GDPR Compliance

- Data encryption at rest
- Audit logging for all data access
- Column-level access control
- Data retention policies

### SOC 2 Compliance

- Access control
- Audit logging
- Encryption
- Incident response

---

## Related Documentation

- [Error Messages Reference](./ERROR_MESSAGES.md)
- [API Reference](./API_REFERENCE.md)
- [Audit System](./AUDIT_SYSTEM.md)