# SQLRustGo 运维手册

> **版本**: 1.0
> **适用版本**: v3.0.0
> **最后更新**: 2026-05-06

---

## 一、部署架构

SQLRustGo 是单进程数据库引擎，支持以下部署模式：

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| 嵌入式 | 嵌入到 Rust 应用中 | 开发测试、IoT |
| 独立服务 | TCP Server + REPL | 生产单机部署 |
| 主从复制 | Semi-sync + GTID | 高可用 |

### 系统要求

| 项目 | 最低要求 | 推荐 |
|------|---------|------|
| CPU | 2 核 | 4 核+ |
| 内存 | 2GB | 8GB+ |
| 磁盘 | 1GB 可用 | SSD 10GB+ |
| Rust | 1.85+ | 1.85+ |

---

## 二、安装与启动

### 从源码安装

```bash
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo
cargo build --release
./target/release/sqlrustgo
```

### Docker（待实现）

```bash
docker pull sqlrustgo:latest
docker run -d -p 4000:4000 -v /data/sqlrustgo:/data sqlrustgo:latest
```

---

## 三、配置

### 配置文件格式（TOML）

```toml
[server]
host = "0.0.0.0"
port = 4000
max_connections = 151

[storage]
data_dir = "/var/lib/sqlrustgo"

[logging]
level = "info"
slow_query_threshold_ms = 1000
slow_query_log = "/var/log/sqlrustgo/slow.log"

[security]
ssl_enabled = false

[vector]
hnsw_m = 16
hnsw_ef_construction = 200

[cache]
query_cache_size = 256
```

### 环境变量覆盖

```bash
SQLRUSTGO_SERVER_PORT=5000 sqlrustgo
```

---

## 四、监控

### 健康检查端点

| 端点 | 说明 |
|------|------|
| `GET /health/live` | Liveness 探针 |
| `GET /health/ready` | Readiness 探针 |
| `GET /metrics` | Prometheus 指标 |

### Prometheus 指标

| 指标 | 类型 | 说明 |
|------|------|------|
| `sqlrustgo_queries_total` | Counter | 总查询数 |
| `sqlrustgo_query_duration_ms` | Histogram | 查询延迟分布 |
| `sqlrustgo_connections_active` | Gauge | 活跃连接数 |
| `sqlrustgo_storage_rows_scanned` | Counter | 扫描行数 |

### Grafana 仪表板

```json
{
  "title": "SQLRustGo Overview",
  "panels": [
    {"title": "QPS", "type": "graph", "target": "sqlrustgo_queries_total"},
    {"title": "Latency P99", "type": "graph", "target": "sqlrustgo_query_duration_ms"}
  ]
}
```

---

## 五、日志

### 慢查询日志

启用方式（配置文件）:
```toml
[logging]
slow_query_threshold_ms = 1000
slow_query_log = "/var/log/sqlrustgo/slow.log"
```

日志格式:
```
# Time: 2026-05-06T10:30:00.000Z
# Query_time: 1.234  Lock_time: 0.000  Rows_sent: 100  Rows_examined: 10000
SELECT * FROM orders WHERE o_orderdate >= '1993-01-01';
```

查看慢查询:
```bash
tail -f /var/log/sqlrustgo/slow.log
```

### 应用日志

| 级别 | 说明 |
|------|------|
| error | 数据库错误和异常 |
| warn | 潜在问题告警 |
| info | 启动、关闭、连接事件 |
| debug | 查询详情、锁信息 |

---

## 六、备份与恢复

### 全量备份

```bash
# 使用内置导出工具
cargo run --bin sqlrustgo-tools -- mysqldump --output backup.sql

# 手动备份数据目录
tar -czf sqlrustgo_backup_$(date +%Y%m%d).tar.gz /var/lib/sqlrustgo/
```

### 增量备份（WAL 归档）

SQLRustGo 使用 Write-Ahead Logging (WAL)。WAL 文件位于数据目录下。

### 恢复

```bash
# 从全量备份恢复
tar -xzf sqlrustgo_backup_20260506.tar.gz -C /var/lib/sqlrustgo/

# 启动后自动重放 WAL
sqlrustgo --data-dir /var/lib/sqlrustgo/
```

---

## 七、性能调优

### Buffer Pool 配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `buffer_pool_size` | 256MB | 数据缓存大小 |
| `buffer_pool_instances` | 4 | 缓存分片数 |

### WAL 配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `wal_buffer_size` | 4MB | WAL 缓冲区 |
| `group_commit_batch_size` | 100 | 批量提交大小 |
| `group_commit_timeout_ms` | 10 | 批提交超时 |

### 查询缓存

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `query_cache_size` | 256MB | 查询结果缓存 |
| `query_cache_enabled` | true | 是否启用 |

---

## 八、故障排查

### 常见问题

| 症状 | 可能原因 | 解决方案 |
|------|---------|----------|
| 连接被拒绝 | 端口未监听 | 检查 `netstat -an | grep 4000` |
| 查询慢 | 无索引 | `CREATE INDEX` |
| 内存溢出 | Buffer Pool 过大 | 减小 `buffer_pool_size` |
| WAL 损坏 | 磁盘损坏 | 运行 `sqlrustgo --wal-recover` |

### 诊断命令

```bash
# 查看数据库状态
SHOW STATUS;

# 查看系统变量
SHOW VARIABLES;

# 分析查询计划
EXPLAIN SELECT * FROM users WHERE id = 1;
```

---

## 九、升级指南

### v2.9.0 → v3.0.0

1. 停止 sqlrustgo 服务
2. 备份数据目录
3. 替换二进制文件
4. 启动服务
5. 运行 `ANALYZE;` 更新统计信息

### 版本兼容性

| 从版本 | 到版本 | 存储格式 |
|--------|--------|----------|
| v2.9.x | v3.0.0 | 兼容 |
| v2.8.x | v2.9.x | 兼容 |

---

## 十、安全

### 用户与权限

```sql
-- 创建角色
CREATE ROLE read_only;

-- 授权
GRANT SELECT ON * TO read_only;

-- 设置角色
SET ROLE read_only;
```

### SSL/TLS 配置

*（v3.0.0 新增，待实现）*

```toml
[security]
ssl_cert = "/etc/sqlrustgo/cert.pem"
ssl_key = "/etc/sqlrustgo/key.pem"
ssl_require = true
```

--- 

*本文档由 SQLRustGo Team 维护。更新日期: 2026-05-06*
