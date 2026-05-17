# v3.2.0 Deployment Guide

> **Version**: 3.2.0
> **Date**: 2026-05-15
> **Status**: Beta Phase

---

## Overview

本指南适用于生产环境部署 SQLRustGo v3.2.0。

### 版本定位

- **Environment**: Production
- **Branch**: `develop/v3.2.0`
- **Status**: Beta Phase

---

## Pre-Deployment Checklist

### Infrastructure

- [ ] CPU: 4+ cores recommended
- [ ] Memory: 8+ GB RAM
- [ ] Disk: SSD with 50+ GB
- [ ] Network: Stable connection

### Configuration

- [ ] `sqlrustgo.toml` configured
- [ ] TLS certificates prepared
- [ ] Backup strategy defined
- [ ] Monitoring configured

### Security

- [ ] Root password changed
- [ ] Firewall configured
- [ ] TLS enabled
- [ ] Audit logging enabled

---

## Deployment Modes

### 1. Standalone Deployment

Single server deployment:

```bash
# Create data directory
mkdir -p /data/sqlrustgo
chown sqlrustgo:sqlrustgo /data/sqlrustgo

# Run
sqlrustgo --config /etc/sqlrustgo/sqlrustgo.toml
```

### 2. Docker Deployment

```bash
# Pull image
docker pull sqlrustgo:v3.2.0

# Create network
docker network create sqlrustgo-net

# Run container
docker run -d \
  --name sqlrustgo \
  --network sqlrustgo-net \
  -p 3306:3306 \
  -v /data/sqlrustgo:/data \
  -v /etc/sqlrustgo:/etc/sqlrustgo \
  sqlrustgo:v3.2.0
```

### 3. Systemd Service

Create `/etc/systemd/system/sqlrustgo.service`:

```ini
[Unit]
Description=SQLRustGo v3.2.0
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo --config /etc/sqlrustgo/sqlrustgo.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
systemctl daemon-reload
systemctl enable sqlrustgo
systemctl start sqlrustgo
```

---

## Configuration Reference

### Server Configuration

```toml
[server]
bind_address = "0.0.0.0"
port = 3306
max_connections = 200
connection_timeout = 30

[database]
path = "/data/sqlrustgo/db"
wal_mode = "write_ahead_log"
page_size = 4096

[performance]
buffer_pool_size = 4096
max_query_workers = 8
statement_timeout = 30000

[gmp]
enable = true
audit_chain_enabled = true
signature_algorithm = "ecdsa"
hsm_provider = "pkcs11"
```

---

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
```

### Metrics

```bash
# Enable metrics endpoint
[monitoring]
enable_metrics = true
metrics_port = 9090
```

---

## Backup & Recovery

### Backup

```bash
# Create backup
BACKUP_DIR=/backup/sqlrustgo-$(date +%Y%m%d)
mkdir -p $BACKUP_DIR

# Backup database
sqlrustgo backup --output $BACKUP_DIR/db.tar.gz

# Backup config
cp /etc/sqlrustgo/sqlrustgo.toml $BACKUP_DIR/

# Backup WAL
cp /data/sqlrustgo/wal/* $BACKUP_DIR/wal/
```

### Recovery

```bash
# Stop service
systemctl stop sqlrustgo

# Restore
tar -xzf /backup/sqlrustgo-20260515/db.tar.gz -C /data/sqlrustgo

# Start service
systemctl start sqlrustgo
```

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Connection refused | Check port 3306 is open |
| OOM | Increase buffer_pool_size |
| Slow queries | Enable query logging |
| WAL corruption | Restore from backup |

### Logs

```bash
# View logs
journalctl -u sqlrustgo -f

# Or
tail -f /var/log/sqlrustgo/sqlrustgo.log
```

---

## Rollback

### Quick Rollback

```bash
# Stop
systemctl stop sqlrustgo

# Restore previous version
docker pull sqlrustgo:v3.1.0
docker tag sqlrustgo:v3.1.0 sqlrustgo:latest

# Restart
systemctl start sqlrustgo
```

---

## Security Checklist

- [ ] TLS enabled
- [ ] Strong passwords configured
- [ ] Firewall restricts port 3306
- [ ] Audit logging enabled
- [ ] Regular backups scheduled
- [ ] Monitoring alerts configured

---

## Next Steps

- Configure [PERFORMANCE_BENCHMARK.md](../v3.1.0/PERFORMANCE_BENCHMARK-zh.md)
- Review [TEST_PLAN.md](./TEST_PLAN.md)
- Read [RELEASE_NOTES.md](./RELEASE_NOTES.md)

---

**Deployment Date**: 2026-05-15
**Maintenance**: hermes-z6g4
