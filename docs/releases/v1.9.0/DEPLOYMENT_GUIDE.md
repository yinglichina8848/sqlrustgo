# SQLRustGo v1.9.0 生产部署指南

> **版本**: v1.9.0  
> **目标**: 单机生产就绪  
> **更新日期**: 2026-03-26

---

## 一、系统要求

### 1.1 硬件要求

| 配置 | 最低要求 | 推荐配置 |
|------|---------|---------|
| CPU | 2 核 | 4+ 核 |
| 内存 | 4 GB | 8+ GB |
| 磁盘 | 10 GB SSD | 50+ GB SSD |
| 系统 | macOS/Linux | Linux (Ubuntu 20.04+) |

### 1.2 软件要求

- Rust 1.70+
- Cargo (随 Rust 安装)
- 可选: PostgreSQL 客户端 (用于测试连接)

---

## 二、快速部署

### 2.1 编译安装

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# Release 编译 (推荐生产环境)
cargo build --release

# 或安装到系统
cargo install --path .
```

### 2.2 启动服务

```bash
# 启动默认服务 (localhost:5678)
cargo run --release

# 自定义端口
SQLRUSTGO_PORT=5679 cargo run --release

# 后台运行
nohup cargo run --release > sqlrustgo.log 2>&1 &
```

### 2.3 验证服务

```bash
# 检查服务健康
curl http://localhost:5678/health

# 或使用 CLI 连接
cargo run --release -- --host localhost --port 5678
```

---

## 三、服务配置

### 3.1 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SQLRUSTGO_PORT` | 5678 | 服务端口 |
| `SQLRUSTGO_HOST` | localhost | 绑定地址 |
| `SQLRUSTGO_DATA_DIR` | ./data | 数据目录 |
| `SQLRUSTGO_BUFFER_POOL_SIZE` | 1000 | 缓冲池大小 |
| `SQLRUSTGO_LOG_LEVEL` | info | 日志级别 |

### 3.2 配置文件

创建 `sqlrustgo.toml`:

```toml
[server]
host = "0.0.0.0"
port = 5678
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo"
buffer_pool_size = 10000

[query_cache]
enabled = true
max_entries = 1000

[logging]
level = "info"
path = "/var/log/sqlrustgo"
```

---

## 四、数据管理

### 4.1 数据目录结构

```
data/
├── catalog/      # 元数据
├── storage/      # 数据文件
├── wal/         # WAL 日志
└── backup/      # 备份文件
```

### 4.2 数据导出

```sql
-- 导出为 CSV
EXPORT TO 'users.csv' FORMAT csv FROM SELECT * FROM users;

-- 导出为 JSON
EXPORT TO 'users.json' FORMAT json FROM SELECT * FROM users;

-- 导出为 SQL
EXPORT TO 'users.sql' FORMAT sql FROM SELECT * FROM users;
```

### 4.3 数据导入

```sql
-- 导入 CSV
IMPORT FROM 'users.csv' FORMAT csv INTO users;

-- 导入 SQL
SOURCE 'users.sql';
```

### 4.4 自动备份

```bash
# 每日备份脚本
#!/bin/bash
DATE=$(date +%Y%m%d)
BACKUP_DIR="/var/backups/sqlrustgo"

# 导出数据
cargo run --release -- export --format sql --output $BACKUP_DIR/db_$DATE.sql

# 压缩
gzip $BACKUP_DIR/db_$DATE.sql

# 保留 7 天
find $BACKUP_DIR -mtime +7 -delete
```

---

## 五、监控运维

### 5.1 健康检查

```bash
# HTTP 健康检查
curl http://localhost:5678/health

# 响应示例
# {"status":"healthy","version":"1.9.0","uptime":3600}
```

### 5.2 性能指标

```sql
-- 查看执行统计
EXPLAIN ANALYZE SELECT * FROM users WHERE id > 100;

-- 查看缓存统计
SELECT * FROM system.query_cache_stats;

-- 查看连接状态
SELECT * FROM system.connections;
```

### 5.3 日志配置

```bash
# 设置日志级别
export SQLRUSTGO_LOG_LEVEL=debug

# 查看实时日志
tail -f sqlrustgo.log
```

日志级别: `error` | `warn` | `info` | `debug` | `trace`

---

## 六、安全配置

### 6.1 网络安全

```toml
[server]
# 只监听本地
host = "127.0.0.1"

# 或指定内网 IP
host = "192.168.1.100"
```

### 6.2 连接限制

```toml
[server]
max_connections = 50
connection_timeout = 30
```

### 6.3 数据安全

- 定期备份数据目录
- 使用 WAL 进行崩溃恢复
- 敏感数据加密存储 (未来版本)

---

## 七、运维脚本

### 7.1 启动脚本

创建 `/etc/init.d/sqlrustgo`:

```bash
#!/bin/bash
NAME="sqlrustgo"
PID_FILE="/var/run/$NAME.pid"
LOG_FILE="/var/log/$NAME.log"

start() {
    echo "Starting $NAME..."
    cd /opt/sqlrustgo
    nohup cargo run --release >> $LOG_FILE 2>&1 &
    echo $! > $PID_FILE
}

stop() {
    echo "Stopping $NAME..."
    if [ -f $PID_FILE ]; then
        kill $(cat $PID_FILE)
        rm $PID_FILE
    fi
}

case "$1" in
    start) start ;;
    stop) stop ;;
    restart) stop; start ;;
    *) echo "Usage: $0 {start|stop|restart}" ;;
esac
```

### 7.2 监控脚本

创建 `monitor.sh`:

```bash
#!/bin/bash

# 检查服务健康
if curl -s http://localhost:5678/health | grep -q "healthy"; then
    echo "OK: Service is healthy"
    exit 0
else
    echo "ERROR: Service is down"
    exit 1
fi
```

加入 crontab:

```bash
*/5 * * * * /opt/sqlrustgo/monitor.sh
```

---

## 八、故障排查

### 8.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 连接失败 | 端口被占用 | 检查端口或更换端口 |
| 启动失败 | 数据目录权限 | 检查目录权限 |
| 性能下降 | 缓冲池不足 | 增加 buffer_pool_size |
| 磁盘空间不足 | 数据过大 | 清理或备份数据 |

### 8.2 崩溃恢复

SQLRustGo 使用 WAL 进行崩溃恢复:

1. 重启服务
2. 系统自动重放 WAL 日志
3. 检查数据完整性

```bash
# 检查日志
tail -f sqlrustgo.log | grep -i recovery
```

### 8.3 数据修复

```sql
-- 检查表完整性
CHECK TABLE users;

-- 重建索引
REINDEX TABLE users;

-- 收集统计信息
ANALYZE TABLE users;
```

---

## 九、性能调优

### 9.1 缓冲池配置

```toml
[storage]
# 根据内存大小配置
# 建议: 内存的 50-70%
buffer_pool_size = 10000
```

### 9.2 查询缓存

```toml
[query_cache]
enabled = true
max_entries = 10000
ttl_seconds = 3600
```

### 9.3 索引优化

```sql
-- 为常用查询创建索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_orders_date ON orders(created_at);
```

---

## 十、升级指南

### 10.1 数据备份

```bash
# 备份数据目录
cp -r /var/lib/sqlrustgo /var/lib/sqlrustgo.backup
```

### 10.2 升级步骤

```bash
# 1. 停止服务
systemctl stop sqlrustgo

# 2. 拉取新版本
git pull origin release/v1.9.0

# 3. 编译
cargo build --release

# 4. 启动服务
systemctl start sqlrustgo

# 5. 验证
curl http://localhost:5678/health
```

---

## 附录

### A. Docker 部署

```dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/sqlrustgo /usr/local/bin/
EXPOSE 5678
CMD ["sqlrustgo"]
```

### B. Docker Compose

```yaml
version: '3.8'
services:
  sqlrustgo:
    build: .
    ports:
      - "5678:5678"
    volumes:
      - ./data:/app/data
    environment:
      - SQLRUSTGO_LOG_LEVEL=info
```

---

*文档版本: v1.9.0*
*更新日期: 2026-03-26*
