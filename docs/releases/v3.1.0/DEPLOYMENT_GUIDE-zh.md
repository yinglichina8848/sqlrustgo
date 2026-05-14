# v3.1.0 部署指南

> **版本**: 3.1.0

---

## 部署架构

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

## 部署模式

### 1. 单节点（开发环境）

```bash
# 最小化部署
./sqlrustgo --data-dir /var/lib/sqlrustgo

# 启用 TLS
./sqlrustgo --data-dir /var/lib/sqlrustgo \
  --tls-cert /etc/sqlrustgo/server.crt \
  --tls-key /etc/sqlrustgo/server.key
```

### 2. 单节点（生产环境）

```bash
# 生产环境配置
./sqlrustgo --config /etc/sqlrustgo/production.toml

# 带有 PID 文件
./sqlrustgo --config /etc/sqlrustgo/production.toml \
  --pid-file /var/run/sqlrustgo.pid
```

### 3. Nomad 部署

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

## 高可用性（半同步）

```bash
# 主节点
./sqlrustgo --config primary.toml

# 副本节点
./sqlrustgo --config replica.toml \
  --replica-source=192.168.0.101:3306
```

---

## 备份与恢复

### 备份

```bash
# 全量备份
./sqlrustgo backup --data-dir /var/lib/sqlrustgo \
  --output /backup/sqlrustgo-$(date +%Y%m%d).tar.gz

# 增量备份（通过 WAL）
./sqlrustgo backup-incremental \
  --data-dir /var/lib/sqlrustgo \
  --wal-dest /backup/wal/
```

### 时间点恢复

```bash
# 停止服务器
pkill sqlrustgo

# 从备份恢复
./sqlrustgo restore \
  --backup /backup/sqlrustgo-20260501.tar.gz \
  --data-dir /var/lib/sqlrustgo

# 恢复到指定时间点
./sqlrustgo recover \
  --data-dir /var/lib/sqlrustgo \
  --until "2026-05-10 12:00:00"
```

---

## 监控

### 指标端点

```bash
# 启用指标
./sqlrustgo --metrics-addr :9090

# Prometheus 抓取配置
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['192.168.0.100:9090']
```

### 关键指标

| 指标 | 描述 | 告警阈值 |
|--------|-------------|----------------|
| `sqlrustgo_queries_total` | 查询总数 | >10K/s |
| `sqlrustgo_query_duration_seconds` | 查询延迟 p99 | >1s |
| `sqlrustgo_storage_used_bytes` | 已用存储 | >80% 容量 |
| `sqlrustgo_wal_lag_seconds` | WAL 延迟 | >5s |
| `sqlrustgo_audit_chain_broken` | 审计链断裂 | ==1 |

---

## 安全检查清单

- [ ] 所有连接启用 TLS
- [ ] 已修改 root 密码（`ALTER USER 'root'@'%' IDENTIFIED BY '...';`）
- [ ] 已删除不必要的用户（`DROP USER 'root'@'::1';`）
- [ ] 防火墙限制 3306 端口仅对可信 IP 开放
- [ ] 数据目录权限设为 `700`
- [ ] 已启用审计日志（`gmp.audit_enabled=true`）
- [ ] 已启用加密（`gmp.encryption_enabled=true`）
- [ ] 定期安全更新

---

## GMP 部署注意事项

要实现 GMP 合规部署：

1. **审计追踪**: 确保 `audit_sha256_required=true`
2. **加密**: 对所有数据页启用 AES-256-GCM 加密
3. **间隙锁**: 确保 `serializable_enabled=true`
4. **备份**: 每日全量备份 + WAL 持续归档
5. **监控**: 审计链断裂时告警
6. **访问控制**: 基于 RBAC 的列级强制执行

---

## 故障排除

| 问题 | 解决方案 |
|-------|----------|
| 慢查询 | 启用 `EXPLAIN ANALYZE`，检查索引使用情况 |
| 内存溢出 | 减小 `buffer_pool.size`，启用查询缓存 |
| 复制延迟 | 检查网络，增加 `replica_parallel_workers` |
| 审计链断裂 | 调查篡改行为，从备份恢复 |
| 加密密钥丢失 | 数据不可恢复 —— 请务必备份密钥 |
