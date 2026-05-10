# SQLRustGo v3.0.0 部署指南

> **版本**: v3.0.0
> **发布日期**: 2026-05-07
> **阶段**: GA (General Availability)

---

## 一、部署概述

### 1.1 部署模式

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| 单机部署 | 单实例运行 | 开发/测试/小规模生产 |
| 主从复制 | 一主多从 Semi-sync 复制 | 中等规模生产 |
| 多源复制 | 多个主源到一个从属 | 数据聚合场景 |

### 1.2 部署前置要求

| 组件 | 要求 |
|------|------|
| 操作系统 | Linux (Ubuntu 22.04+) / macOS 13+ |
| CPU | 2+ 核心 |
| 内存 | 4+ GB |
| 磁盘 | 10+ GB |

---

## 二、单机部署

### 2.1 Release 构建

```bash
# 构建 Release 版本
cargo build --all-features --release

# 验证
./target/release/sqlrustgo --version
# 输出: sqlrustgo 3.0.0
```

### 2.2 配置目录

```bash
# 创建数据目录
sudo mkdir -p /var/lib/sqlrustgo
sudo chown $USER:$USER /var/lib/sqlrustgo

# 创建配置目录
sudo mkdir -p /etc/sqlrustgo
```

### 2.3 配置文件

创建 `/etc/sqlrustgo/sqlrustgo.toml`:

```toml
[server]
host = "0.0.0.0"
port = 15995
max_connections = 100
connection_pool_size = 10

[storage]
data_dir = "/var/lib/sqlrustgo/data"
wal_dir = "/var/lib/sqlrustgo/wal"
buffer_pool_size = 1000  # 页数

[transaction]
wal_sync = "full"  # "full", "partial", "none"
checkpoint_interval = 300  # 秒

[replication]
mode = "none"  # "none", "semi-sync", "multi-source"

[logging]
level = "info"  # "debug", "info", "warn", "error"
log_dir = "/var/log/sqlrustgo"
```

### 2.4 启动服务

```bash
# 前台启动 (测试用)
./target/release/sqlrustgo --config /etc/sqlrustgo/sqlrustgo.toml

# 后台启动
nohup ./target/release/sqlrustgo --config /etc/sqlrustgo/sqlrustgo.toml \
  > /var/log/sqlrustgo/sqlrustgo.log 2>&1 &

# 验证启动
curl http://localhost:15995/status
```

### 2.5 systemd 服务

创建 `/etc/systemd/system/sqlrustgo.service`:

```ini
[Unit]
Description=SQLRustGo v3.0.0
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
WorkingDirectory=/var/lib/sqlrustgo
ExecStart=/opt/sqlrustgo/bin/sqlrustgo --config /etc/sqlrustgo/sqlrustgo.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

注册并启动:

```bash
# 创建服务用户
sudo useradd -r -s /bin/false sqlrustgo

# 设置权限
sudo chown -R sqlrustgo:sqlrustgo /var/lib/sqlrustgo /var/log/sqlrustgo

# 重新加载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl enable sqlrustgo
sudo systemctl start sqlrustgo

# 检查状态
sudo systemctl status sqlrustgo
```

---

## 三、主从复制部署

### 3.1 架构

```
┌─────────────┐      Semi-sync       ┌─────────────┐
│   Master    │ ──────────────────▶  │   Slave 1   │
│  (Primary)  │                      │  (Replica)  │
└─────────────┘                      └─────────────┘
       │
       │  Multi-source
       ▼
┌─────────────┐
│   Slave 2   │
│ (Aggregator)│
└─────────────┘
```

### 3.2 Master 配置

```toml
[server]
host = "0.0.0.0"
port = 15995

[replication]
mode = "semi-sync"
server_id = 1
```

### 3.3 Slave 配置

```toml
[server]
host = "0.0.0.0"
port = 15996

[replication]
mode = "semi-sync"
server_id = 2
master_host = "master.example.com"
master_port = 15995
```

### 3.4 启动复制

```sql
-- 在 Master 上创建复制用户
CREATE USER 'repl'@'%' IDENTIFIED BY 'repl_password';
GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';

-- 在 Slave 上启动复制
CHANGE MASTER TO
    MASTER_HOST = 'master.example.com',
    MASTER_PORT = 15995,
    MASTER_USER = 'repl',
    MASTER_PASSWORD = 'repl_password',
    MASTER_AUTO_POSITION = 1;

START SLAVE;

-- 检查复制状态
SHOW SLAVE STATUS\G
```

---

## 四、Docker 部署

### 4.1 Dockerfile

```dockerfile
FROM rust:1.85 as builder

WORKDIR /build
COPY . .
RUN cargo build --all-features --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/sqlrustgo /usr/local/bin/
EXPOSE 15995
CMD ["sqlrustgo"]
```

### 4.2 Docker Compose

```yaml
version: '3.8'

services:
  sqlrustgo:
    build: .
    ports:
      - "15995:15995"
    volumes:
      - sqlrustgo_data:/var/lib/sqlrustgo
      - ./config/sqlrustgo.toml:/etc/sqlrustgo/sqlrustgo.toml:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped

volumes:
  sqlrustgo_data:
```

启动:

```bash
docker-compose up -d
docker-compose logs -f sqlrustgo
```

---

## 五、Kubernetes 部署

### 5.1 StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: sqlrustgo
spec:
  serviceName: sqlrustgo
  replicas: 1
  selector:
    matchLabels:
      app: sqlrustgo
  template:
    metadata:
      labels:
        app: sqlrustgo
    spec:
      containers:
        - name: sqlrustgo
          image: sqlrustgo:v3.0.0
          ports:
            - containerPort: 15995
          volumeMounts:
            - name: data
              mountPath: /var/lib/sqlrustgo
          resources:
            requests:
              memory: "2Gi"
              cpu: "500m"
            limits:
              memory: "4Gi"
              cpu: "2000m"
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
```

### 5.2 Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo
spec:
  selector:
    app: sqlrustgo
  ports:
    - port: 15995
      targetPort: 15995
  type: ClusterIP
```

---

## 六、运维操作

### 6.1 备份

```bash
# 创建备份目录
BACKUP_DIR=/var/backups/sqlrustgo
mkdir -p $BACKUP_DIR

# 备份数据
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
tar -czf $BACKUP_DIR/sqlrustgo_data_$TIMESTAMP.tar.gz \
    /var/lib/sqlrustgo/data

# 备份 WAL
tar -czf $BACKUP_DIR/sqlrustgo_wal_$TIMESTAMP.tar.gz \
    /var/lib/sqlrustgo/wal

# 保留最近 7 天备份
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

### 6.2 恢复

```bash
# 停止服务
systemctl stop sqlrustgo

# 恢复数据
tar -xzf /var/backups/sqlrustgo/sqlrustgo_data_20260510.tar.gz \
    -C /

# 重启服务
systemctl start sqlrustgo
```

### 6.3 升级

```bash
# 1. 停止服务
systemctl stop sqlrustgo

# 2. 备份
bash scripts/backup.sh

# 3. 更新代码
git fetch origin
git checkout v3.0.0

# 4. 重新构建
cargo build --all-features --release

# 5. 启动服务
systemctl start sqlrustgo

# 6. 验证
curl http://localhost:15995/status
```

---

## 七、监控

### 7.1 健康检查

```bash
# HTTP 健康检查
curl http://localhost:15995/health

# SQL 状态查询
./sqlrustgo -e "SELECT version();"
```

### 7.2 日志监控

```bash
# 实时查看日志
tail -f /var/log/sqlrustgo/sqlrustgo.log

# 错误日志
grep -i error /var/log/sqlrustgo/sqlrustgo.log

# 慢查询 (假设慢查询阈值为 1 秒)
grep -i "slow query" /var/log/sqlrustgo/sqlrustgo.log
```

### 7.3 性能指标

```sql
-- 连接状态
SHOW STATUS LIKE 'Threads_connected';

-- 缓存命中率
SHOW STATUS LIKE 'Buffer_pool_hit_rate';

-- QPS
SHOW STATUS LIKE 'Queries_per_second';
```

---

## 八、安全

### 8.1 用户权限

```sql
-- 创建管理员用户
CREATE USER 'admin'@'localhost' IDENTIFIED BY 'strong_password';
GRANT ALL PRIVILEGES ON *.* TO 'admin'@'localhost';

-- 创建只读用户
CREATE USER 'readonly'@'%' IDENTIFIED BY 'readonly_password';
GRANT SELECT ON *.* TO 'readonly'@'%';

-- 刷新权限
FLUSH PRIVILEGES;
```

### 8.2 网络安全

```toml
[server]
# 仅监听本地
host = "127.0.0.1"
port = 15995

# 或使用防火墙限制
# iptables -A INPUT -p tcp --dport 15995 -s 10.0.0.0/8 -j ACCEPT
```

---

## 九、故障排除

| 问题 | 解决方案 |
|------|----------|
| 服务启动失败 | 检查端口占用 `netstat -tlnp \| grep 15995` |
| 连接超时 | 检查防火墙和 `bind_address` 配置 |
| 数据不一致 | 检查主从复制状态 `SHOW SLAVE STATUS` |
| 性能下降 | 检查 buffer pool 大小和慢查询日志 |

---

## 相关文档

- [安装指南](./INSTALL.md)
- [快速开始](./QUICK_START.md)
- [性能目标](./PERFORMANCE_TARGETS.md)

---

*最后更新: 2026-05-07*
