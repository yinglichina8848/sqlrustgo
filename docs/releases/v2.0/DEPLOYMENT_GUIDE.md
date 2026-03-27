# SQLRustGo v2.0 部署与运维手册

> **版本**: 2.0
> **日期**: 2026-03-26
> **状态**: 规划中

---

## 1. 系统要求

### 1.1 硬件要求

| 配置 | 最小 | 推荐 | 生产环境 |
|------|------|------|----------|
| CPU | 2 核 | 4 核 | 8+ 核 |
| 内存 | 2 GB | 4 GB | 16+ GB |
| 磁盘 | 10 GB | 50 GB | 100+ GB SSD |
| 网络 | 100 Mbps | 1 Gbps | 10 Gbps |

### 1.2 软件要求

- **操作系统**: Linux (Ubuntu 20.04+, CentOS 8+), macOS 12+, Windows 10+
- **Rust**: 1.75.0+
- **依赖**: OpenSSL (用于 SSL/TLS)

---

## 2. 安装部署

### 2.1 从源码编译

```bash
# 克隆代码
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 编译
cargo build --release

# 安装
cargo install --path .
```

### 2.2 Docker 部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  sqlrustgo:
    image: sqlrustgo:latest
    ports:
      - "3306:3306"
    volumes:
      - ./data:/data
      - ./config:/etc/sqlrustgo
    environment:
      - SQLRUSTGO_DATA_DIR=/data
      - SQLRUSTGO_PORT=3306
    restart: unless-stopped
```

---

## 3. 配置说明

### 3.1 配置文件结构

```toml
# sqlrustgo.toml
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
engine = "memory"  # 或 "file"
data_dir = "/data/sqlrustgo"

[buffer_pool]
size_mb = 256

[logging]
level = "info"
path = "/var/log/sqlrustgo"

[replication]
enabled = false
```

### 3.2 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| server.host | String | "0.0.0.0" | 监听地址 |
| server.port | u16 | 3306 | 监听端口 |
| server.max_connections | u32 | 100 | 最大连接数 |
| storage.engine | String | "memory" | 存储引擎 |
| buffer_pool.size_mb | u64 | 256 | Buffer Pool 大小 |
| logging.level | String | "info" | 日志级别 |

---

## 4. 启动与停止

### 4.1 启动服务

```bash
# 前台启动
sqlrustgo --config sqlrustgo.toml

# 后台启动
sqlrustgo --config sqlrustgo.toml --daemon

# 使用 systemd
sudo systemctl start sqlrustgo
```

### 4.2 停止服务

```bash
# 正常停止
sqlrustgo --shutdown

# 使用 systemd
sudo systemctl stop sqlrustgo
```

### 4.3 健康检查

```bash
# HTTP 健康检查
curl http://localhost:8080/health

# MySQL 协议检查
mysql -h localhost -P 3306 -u root -p -e "SELECT 1"
```

---

## 5. 日志管理

### 5.1 日志位置

| 日志类型 | 默认路径 |
|----------|----------|
| 错误日志 | /var/log/sqlrustgo/error.log |
| 查询日志 | /var/log/sqlrustgo/query.log |
| 慢查询日志 | /var/log/sqlrustgo/slow.log |
| 审计日志 | /var/log/sqlrustgo/audit.log |

### 5.2 日志级别

| 级别 | 说明 |
|------|------|
| error | 仅错误 |
| warn | 警告及以上 |
| info | 信息及以上 |
| debug | 调试及以上 |
| trace | 详细跟踪 |

---

## 6. 监控指标

### 6.1 Prometheus 指标

```
# 端点: http://localhost:9090/metrics

# 基础指标
sqlrustgo_up{}                    # 服务是否运行
sqlrustgo_connections_active{}    # 当前活跃连接数
sqlrustgo_queries_total{}         # 总查询数
sqlrustgo_transactions_total{}    # 总事务数

# 性能指标
sqlrustgo_query_duration_seconds{}     # 查询延迟
sqlrustgo_storage_rows_read{}          # 读取行数
sqlrustgo_storage_rows_written{}        # 写入行数

# 缓存指标
sqlrustgo_cache_hits_total{}           # 缓存命中
sqlrustgo_cache_misses_total{}          # 缓存未命中
```

### 6.2 关键告警阈值

| 指标 | 告警阈值 | 严重程度 |
|------|----------|----------|
| 连接数 | >90% 最大连接 | Warning |
| 查询延迟 P99 | >100ms | Warning |
| 内存使用 | >80% | Warning |
| 磁盘使用 | >90% | Critical |
| 错误率 | >0.1% | Critical |

---

## 7. 备份与恢复

### 7.1 备份

```bash
# 全量备份
sqlrustgo-backup --full --output /backup/

# 增量备份
sqlrustgo-backup --incremental --since /backup/full

# 备份到指定路径
sqlrustgo-backup --output /backup/$(date +%Y%m%d)
```

### 7.2 恢复

```bash
# 恢复
sqlrustgo-restore --from /backup/20260326

# 恢复并指定数据目录
sqlrustgo-restore --from /backup/20260326 --data-dir /new/data
```

---

## 8. 主从复制

### 8.1 配置主节点

```toml
[replication]
enabled = true
role = "master"
server_id = 1
binlog_enabled = true
```

### 8.2 配置从节点

```toml
[replication]
enabled = true
role = "slave"
server_id = 2
master_host = "master.example.com"
master_port = 3306
master_user = "repl"
master_password = "password"
```

### 8.3 复制管理命令

```sql
-- 查看复制状态
SHOW REPLICA STATUS;

-- 启动复制
START REPLICA;

-- 停止复制
STOP REPLICA;

-- 手动切换主节点
CHANGE MASTER TO MASTER_HOST='new_master';
```

---

## 9. 故障排查

### 9.1 常见问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 服务启动失败 | 端口被占用 | 检查端口或修改配置 |
| 连接被拒绝 | 权限问题 | 检查用户权限配置 |
| 查询超时 | 资源不足 | 增加资源或优化查询 |
| 磁盘空间不足 | 数据过大 | 清理或扩容 |

### 9.2 诊断命令

```bash
# 查看进程状态
ps aux | grep sqlrustgo

# 查看资源使用
top -p $(pgrep sqlrustgo)

# 查看网络连接
netstat -an | grep 3306

# 查看错误日志
tail -f /var/log/sqlrustgo/error.log
```

---

## 10. 升级流程

### 10.1 小版本升级 (v2.0.x)

```bash
# 1. 停止服务
sudo systemctl stop sqlrustgo

# 2. 备份数据
sqlrustgo-backup --full --output /backup/pre-upgrade

# 3. 更新代码
git pull

# 4. 重新编译
cargo build --release

# 5. 重启服务
sudo systemctl start sqlrustgo
```

### 10.2 大版本升级 (v2.0 -> v2.1)

```bash
# 1. 阅读升级指南
# 2. 检查兼容性
sqlrustgo-upgrade-check --from v2.0 --to v2.1

# 3. 执行升级
# (遵循具体升级文档)
```

---

## 11. 性能调优

### 11.1 Buffer Pool 配置

```toml
[buffer_pool]
# 建议设置为可用内存的 50-70%
size_mb = 4096
```

### 11.2 连接池配置

```toml
[connection_pool]
max_connections = 100
min_idle = 10
idle_timeout_seconds = 600
```

---

**文档版本历史**

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-03-26 | 初始版本 |

**状态**: ✅ 规划完成
