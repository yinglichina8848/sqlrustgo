# v3.1.0 Deployment Guide

> **Version**: 3.1.0

---

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SQLRustGo v3.1.0                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐  │
│  │  MySQL      │     │  Nomad      │     │  Monitoring  │  │
│  │  Clients    │────▶│  Runner     │────▶│  Prometheus  │  │
│  └─────────────┘     └──────┬──────┘     └─────────────┘  │
│                             │                               │
│                    ┌────────▼────────┐                      │
│                    │  SQLRustGo     │                      │
│                    │  Server        │                      │
│                    │  (v3.1.0)     │                      │
│                    └──────┬────────┘                      │
│                             │                               │
│         ┌──────────────────┼──────────────────┐             │
│         │                  │                  │             │
│  ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐    │
│  │  Storage     │   │  WAL        │   │  Audit Log   │    │
│  │  (AES-256)  │   │  (Crash-safe)│   │  (SHA-256)  │    │
│  └─────────────┘   └─────────────┘   └─────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Deployment Modes

### 1. Single Node (Development)

```bash
# Minimal deployment
./sqlrustgo --data-dir /var/lib/sqlrustgo

# With TLS
./sqlrustgo --data-dir /var/lib/sqlrustgo \
  --tls-cert /etc/sqlrustgo/server.crt \
  --tls-key /etc/sqlrustgo/server.key
```

### 2. Single Node (Production)

```bash
# Production config
./sqlrustgo --config /etc/sqlrustgo/production.toml

# With PID file
./sqlrustgo --config /etc/sqlrustgo/production.toml \
  --pid-file /var/run/sqlrustgo.pid
```

### 3. Nomad Deployment

```hcl
# job.sqlrustgo.nomad
job "sqlrustgo" {
  datacenters = ["dc1"]
  type = "service"

  group "sqlrustgo" {
    count = 1

    task "sqlrustgo" {
      driver = "docker"

      config {
        image = "ghcr.io/minzuuniversity/sqlrustgo:v3.1.0"
        ports = ["mysql"]
      }

      env {
        SQLRUSTGO_DATA_DIR = "/var/lib/sqlrustgo"
        SQLRUSTGO_TLS_ENABLED = "true"
      }

      resources {
        cpu    = 2048
        memory = 4096
        network {
          port "mysql" {
            static = 3306
          }
        }
      }
    }
  }
}
```

---

## High Availability (Semi-sync)

```bash
# Primary node
./sqlrustgo --config primary.toml

# Replica node
./sqlrustgo --config replica.toml \
  --replica-source=192.168.0.101:3306
```

---

## Backup & Recovery

### Backup

```bash
# Full backup
./sqlrustgo backup --data-dir /var/lib/sqlrustgo \
  --output /backup/sqlrustgo-$(date +%Y%m%d).tar.gz

# Incremental backup (via WAL)
./sqlrustgo backup-incremental \
  --data-dir /var/lib/sqlrustgo \
  --wal-dest /backup/wal/
```

### Point-in-Time Recovery

```bash
# Stop server
pkill sqlrustgo

# Restore from backup
./sqlrustgo restore \
  --backup /backup/sqlrustgo-20260501.tar.gz \
  --data-dir /var/lib/sqlrustgo

# Recover to point in time
./sqlrustgo recover \
  --data-dir /var/lib/sqlrustgo \
  --until "2026-05-10 12:00:00"
```

---

## Monitoring

### Metrics Endpoint

```bash
# Enable metrics
./sqlrustgo --metrics-addr :9090

# Prometheus scrape config
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['192.168.0.100:9090']
```

### Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|----------------|
| `sqlrustgo_queries_total` | Total queries | >10K/s |
| `sqlrustgo_query_duration_seconds` | Query latency p99 | >1s |
| `sqlrustgo_storage_used_bytes` | Storage used | >80% capacity |
| `sqlrustgo_wal_lag_seconds` | WAL lag | >5s |
| `sqlrustgo_audit_chain_broken` | Audit chain broken | ==1 |

---

## Security Checklist

- [ ] TLS enabled for all connections
- [ ] Root password changed (`ALTER USER 'root'@'%' IDENTIFIED BY '...';`)
- [ ] Unnecessary users removed (`DROP USER 'root'@'::1';`)
- [ ] Firewall restricts port 3306 to trusted IPs
- [ ] Data directory permissions `700`
- [ ] Audit log enabled (`gmp.audit_enabled=true`)
- [ ] Encryption enabled (`gmp.encryption_enabled=true`)
- [ ]定期安全更新

---

## GMP Deployment Notes

For GMP-compliant deployment:

1. **Audit Trail**: Ensure `audit_sha256_required=true`
2. **Encryption**: Enable AES-256-GCM for all data pages
3. **Gap Locking**: Ensure `serializable_enabled=true`
4. **Backup**: Daily full backup + WAL continuous archiving
5. **Monitoring**: Alert on audit chain breaks
6. **Access Control**: RBAC with column-level enforcement

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Slow queries | Enable `EXPLAIN ANALYZE`, check index usage |
| OOM | Reduce `buffer_pool.size`, enable query cache |
| Replication lag | Check network, increase `replica_parallel_workers` |
| Audit chain broken | Investigate tampering, restore from backup |
| Encryption key lost | Data unrecoverable — maintain key backups |
