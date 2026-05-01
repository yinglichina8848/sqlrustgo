# SQLRustGo v2.8.0 部署指南

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **最后更新**: 2026-04-30

---

## 概述

本文档提供 SQLRustGo v2.8.0 的部署指南，包括环境要求、配置选项和部署步骤。v2.8.0 是生产化+分布式+安全版本，新增了分区表、主从复制、GTID、读写分离、SIMD 向量化、列级权限等关键功能。

---

## 1. 环境要求

### 1.1 硬件要求

| 组件 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 4+ 核 (AVX2/AVX-512 支持 SIMD) |
| 内存 | 4 GB | 8+ GB |
| 磁盘 | 10 GB SSD | 50+ GB SSD |

### 1.2 软件要求

- **操作系统**: Linux (Ubuntu 20.04+, CentOS 8+), macOS 12+, Windows 10+ (WSL2)
- **Rust**: 1.85.0+
- **Cargo**: 最新稳定版
- **依赖**: OpenSSL (Linux), CMake 3.15+ (编译依赖)

### 1.3 SIMD 要求

v2.8.0 支持 SIMD 向量化加速:

| SIMD 级别 | 检测结果 | 性能提升 |
|-----------|----------|----------|
| AVX-512 | `detect_simd_lanes() = 16` | 最高 |
| AVX2 | `detect_simd_lanes() = 8` | 高 |
| Fallback | `detect_simd_lanes() = 1` | 标量 |

```bash
# 检测 CPU SIMD 能力
cat /proc/cpuinfo | grep -E "avx|avx2|avx512"
```

---

## 2. 从源码编译

### 2.1 克隆项目

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.8.0 分支
git checkout develop/v2.8.0
```

### 2.2 编译

```bash
# Debug 编译 (快速开发)
cargo build

# Release 编译 (生产部署)
cargo build --release --all-features

# 仅编译 MySQL 协议服务器
cargo build --release -p sqlrustgo-mysql-server

# 仅编译 REST API 服务器
cargo build --release -p sqlrustgo-server
```

### 2.3 安装

```bash
# 安装到系统路径
cargo install --path . --bins

# 或手动安装二进制
chmod +x target/release/sqlrustgo-mysql-server
sudo mv target/release/sqlrustgo-mysql-server /usr/local/bin/

# 验证安装
sqlrustgo-mysql-server --version
```

---

## 3. 快速启动

### 3.1 启动 MySQL 协议服务器

```bash
# 默认配置启动 (127.0.0.1:3306)
sqlrustgo-mysql-server

# 指定配置启动
sqlrustgo-mysql-server --config /etc/sqlrustgo/config.toml

# Docker 启动
docker run -d \
  --name sqlrustgo \
  -p 3306:3306 \
  -p 8080:8080 \
  minzuuniversity/sqlrustgo:v2.8.0
```

### 3.2 启动 REST API 服务器

```bash
# REST API 服务器 (127.0.0.1:8080)
sqlrustgo-server

# 指定端口
sqlrustgo-server --http-port 8080 --mysql-port 3306
```

### 3.3 使用 MySQL 客户端连接

```bash
# 基本连接
mysql -h 127.0.0.1 -P 3306 -u root

# 交互式操作
mysql -h 127.0.0.1 -P 3306 -u root -p
```

---

## 4. 系统服务配置

### 4.1 Linux systemd 服务

```bash
# 创建系统用户
sudo useradd -r -s /bin/false sqlrustgo

# 创建数据目录
sudo mkdir -p /var/lib/sqlrustgo
sudo chown sqlrustgo:sqlrustgo /var/lib/sqlrustgo

# 创建服务文件 /etc/systemd/system/sqlrustgo.service
```

```ini
[Unit]
Description=SQLRustGo Database v2.8.0
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo-mysql-server --config /etc/sqlrustgo/config.toml
Restart=always
RestartSec=5
PermissionsStartOnly=true

# 环境配置
Environment="RUST_LOG=info"
Environment="SQLRUSTGO_DATA_DIR=/var/lib/sqlrustgo"

# 资源限制
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

```bash
# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable sqlrustgo
sudo systemctl start sqlrustgo

# 查看状态
sudo systemctl status sqlrustgo
```

### 4.2 日志配置

```bash
# 查看日志
journalctl -u sqlrustgo -f

# 查看错误日志
journalctl -u sqlrustgo -p err
```

---

## 5. Docker 部署

### 5.1 Docker Compose 单节点

```yaml
# docker-compose.yml
version: '3.8'

services:
  sqlrustgo:
    image: minzuuniversity/sqlrustgo:v2.8.0
    container_name: sqlrustgo
    ports:
      - "3306:3306"   # MySQL 协议
      - "8080:8080"   # REST API
    volumes:
      - sqlrustgo-data:/data
      - ./config.toml:/etc/sqlrustgo/config.toml:ro
    environment:
      - SQLRUSTGO_MODE=production
      - SQLRUSTGO_MAX_CONNECTIONS=100
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost", "-P", "3306"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

volumes:
  sqlrustgo-data:
    driver: local
```

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f sqlrustgo

# 查看状态
docker-compose ps
```

### 5.2 Docker Compose 主从复制集群

```yaml
# docker-compose-replication.yml
version: '3.8'

services:
  primary:
    image: minzuuniversity/sqlrustgo:v2.8.0
    container_name: sqlrustgo-primary
    ports:
      - "3306:3306"
    volumes:
      - primary-data:/data
      - ./config-primary.toml:/etc/sqlrustgo/config.toml:ro
    environment:
      - SQLRUSTGO_MODE=production
      - SQLRUSTGO_REPLICATION_ROLE=primary
    restart: unless-stopped

  replica1:
    image: minzuuniversity/sqlrustgo:v2.8.0
    container_name: sqlrustgo-replica1
    ports:
      - "3307:3306"
    volumes:
      - replica1-data:/data
      - ./config-replica1.toml:/etc/sqlrustgo/config.toml:ro
    environment:
      - SQLRUSTGO_MODE=production
      - SQLRUSTGO_REPLICATION_ROLE=replica
      - SQLRUSTGO_PRIMARY_ADDRESS=primary:3306
    depends_on:
      - primary
    restart: unless-stopped

  replica2:
    image: minzuuniversity/sqlrustgo:v2.8.0
    container_name: sqlrustgo-replica2
    ports:
      - "3308:3306"
    volumes:
      - replica2-data:/data
      - ./config-replica2.toml:/etc/sqlrustgo/config.toml:ro
    environment:
      - SQLRUSTGO_MODE=production
      - SQLRUSTGO_REPLICATION_ROLE=replica
      - SQLRUSTGO_PRIMARY_ADDRESS=primary:3306
    depends_on:
      - primary
    restart: unless-stopped

  # 负载均衡器 (HAProxy)
  haproxy:
    image: haproxy:2.8
    container_name: sqlrustgo-lb
    ports:
      - "3309:3306"
    volumes:
      - ./haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    depends_on:
      - primary
      - replica1
      - replica2
    restart: unless-stopped

volumes:
  primary-data:
  replica1-data:
  replica2-data:
```

### 5.3 主节点配置 (config-primary.toml)

```toml
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/data"
max_memory_mb = 2048

[replication]
enabled = true
role = "primary"
sync_mode = "semi-sync"  # semi-sync 或 async

[logging]
level = "info"
output = "stdout"
```

### 5.4 从节点配置 (config-replica.toml)

```toml
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/data"
max_memory_mb = 2048

[replication]
enabled = true
role = "replica"
primary_address = "primary:3306"
sync_mode = "semi-sync"

[logging]
level = "info"
output = "stdout"
```

---

## 6. Kubernetes 部署

### 6.1 StatefulSet (主从复制)

```yaml
# sqlrustgo-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: sqlrustgo
  labels:
    app: sqlrustgo
spec:
  serviceName: sqlrustgo
  replicas: 3
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
        image: minzuuniversity/sqlrustgo:v2.8.0
        ports:
        - containerPort: 3306
          name: mysql
        - containerPort: 8080
          name: http
        env:
        - name: SQLRUSTGO_MODE
          value: "production"
        - name: SQLRUSTGO_REPLICATION_ROLE
          valueFrom:
            fieldRef:
              fieldPath: metadata.labels['role']
        - name: SQLRUSTGO_MAX_CONNECTIONS
          value: "100"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2"
        volumeMounts:
        - name: data
          mountPath: /data
        livenessProbe:
          tcpSocket:
            port: 3306
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command: ["mysqladmin", "ping", "-h", "localhost"]
          initialDelaySeconds: 5
          periodSeconds: 5
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 50Gi
```

### 6.2 Service (读写分离路由)

```yaml
# sqlrustgo-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-primary
spec:
  type: ClusterIP
  selector:
    role: primary
  ports:
  - name: mysql
    port: 3306
    targetPort: 3306

---
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-replica
spec:
  type: ClusterIP
  # 负载均衡 SELECT 到从节点
  selector:
    role: replica
  ports:
  - name: mysql
    port: 3306
    targetPort: 3306

---
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-lb
spec:
  type: LoadBalancer
  # 负载均衡器，自动路由读写
  selector:
    app: sqlrustgo
  ports:
  - name: mysql
    port: 3306
    targetPort: 3306
```

---

## 7. 配置参考

### 7.1 完整配置示例

```toml
# /etc/sqlrustgo/config.toml

[server]
host = "0.0.0.0"
port = 3306
max_connections = 100
connection_timeout = 30
wait_timeout = 28800

[storage]
data_dir = "/var/lib/sqlrustgo"
temp_dir = "/tmp/sqlrustgo"
max_memory_mb = 4096

[storage.engine]
type = "buffer_pool"
max_buffer_pages = 131072

[logging]
level = "info"  # trace, debug, info, warn, error
output = "stdout"  # stdout, file
file_path = "/var/log/sqlrustgo/server.log"

[security]
enable_auth = true
tls_enabled = false
tls_cert = "/etc/sqlrustgo/server.crt"
tls_key = "/etc/sqlrustgo/server.key"

[privilege]
column_level = true  # 列级权限控制 (T-17)

[audit]
enabled = true
log_dir = "/var/log/sqlrustgo/audit"
log_format = "json"

[replication]  # v2.8.0 分布式特性
enabled = false
role = "primary"  # primary, replica
sync_mode = "semi-sync"
gtid_enabled = true
primary_address = ""

[partition]
enabled = true  # 分区表支持

[simd]  # v2.8.0 SIMD 向量化
enabled = true
auto_detect = true
fallback = "scalar"
```

### 7.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_HOST` | 监听地址 | 0.0.0.0 |
| `SQLRUSTGO_PORT` | MySQL 端口 | 3306 |
| `SQLRUSTGO_HTTP_PORT` | REST API 端口 | 8080 |
| `SQLRUSTGO_DATA_DIR` | 数据目录 | /var/lib/sqlrustgo |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 | info |
| `SQLRUSTGO_MAX_CONNECTIONS` | 最大连接数 | 100 |
| `SQLRUSTGO_MODE` | 运行模式 | development |
| `SQLRUSTGO_REPLICATION_ROLE` | 复制角色 | primary |
| `SQLRUSTGO_PRIMARY_ADDRESS` | 主节点地址 | "" |
| `RUST_LOG` | Rust 日志级别 | info |

---

## 8. 主从复制配置

### 8.1 GTID 复制协议

v2.8.0 支持 GTID (Global Transaction ID) 复制:

```toml
[replication]
enabled = true
role = "primary"
gtid_enabled = true
sync_mode = "semi-sync"

[replication]
enabled = true
role = "replica"
gtid_enabled = true
primary_address = "192.168.1.100:3306"
```

### 8.2 复制状态查询

```sql
-- 查看复制状态
SHOW MASTER STATUS;
SHOW SLAVE STATUS\G;

-- 查看 GTID
SELECT @@GLOBAL.gtid_executed;
SELECT @@GLOBAL.gtid_purged;
```

### 8.3 故障转移配置

```toml
[failover]
enabled = true
detection_interval_ms = 1000
max_detection_ms = 5000
auto_switch = true
switch_timeout_ms = 30000
```

---

## 9. 分区表配置

### 9.1 RANGE 分区

```sql
CREATE TABLE sales (
    id INT,
    sale_date DATE
)
PARTITION BY RANGE (YEAR(sale_date)) (
    PARTITION p2024 VALUES LESS THAN (2025),
    PARTITION p2025 VALUES LESS THAN (2026),
    PARTITION p2026 VALUES LESS THAN MAXVALUE
);
```

### 9.2 LIST 分区

```sql
CREATE TABLE employees (
    id INT,
    department INT
)
PARTITION BY LIST (department) (
    PARTITION p_sales VALUES IN (1, 2, 3),
    PARTITION p_engineering VALUES IN (4, 5, 6),
    PARTITION p_hr VALUES IN (7, 8)
);
```

### 9.3 HASH 分区

```sql
CREATE TABLE orders (
    id INT,
    customer_id INT
)
PARTITION BY HASH (customer_id)
PARTITIONS 8;
```

---

## 10. 备份与恢复

### 10.1 备份

```bash
# 全量备份
sqlrustgo backup --output /backup/sqlrustgo-$(date +%Y%m%d).dump

# 备份到指定目录
sqlrustgo backup --output /backup/full-backup --format=directory

# 仅备份表结构
sqlrustgo backup --output /backup/schema-only --schema-only
```

### 10.2 恢复

```bash
# 从备份恢复
sqlrustgo restore --input /backup/sqlrustgo-20260430.dump

# Point-in-time 恢复
sqlrustgo restore --input /backup/full-backup --point-in-time="2026-04-30 15:00:00"
```

### 10.3 WAL 备份

```bash
# 备份 WAL
sqlrustgo wal backup --output /backup/wal

# 从 WAL 恢复
sqlrustgo wal restore --input /backup/wal
```

---

## 11. 监控

### 11.1 健康检查

```bash
# HTTP 健康检查
curl http://localhost:8080/health

# MySQL 协议 ping
mysqladmin ping -h localhost -P 3306

# 就绪检查
curl http://localhost:8080/ready
```

### 11.2 Prometheus 指标

```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"
```

```bash
# 获取指标
curl http://localhost:9090/metrics

# 关键指标
# sqlrustgo_queries_total - SQL 执行总数
# sqlrustgo_query_duration_seconds - 查询延迟
# sqlrustgo_connections_active - 活跃连接数
# sqlrustgo_replication_lag_seconds - 复制延迟
```

---

## 12. 负载均衡

### 12.1 HAProxy 配置

```ini
# /etc/haproxy/haproxy.cfg
global
    log stdout format raw local0
    maxconn 4096

defaults
    log global
    mode tcp
    option tcplog
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

# 读写分离: 写请求到主节点, 读请求到从节点
frontend mysql-front
    bind *:3306
    default_backend mysql-primary

    # 简单负载均衡 (所有请求到主节点)
    use_backend mysql-primary if { pkg(0) == 0x03 }  # COM_QUERY

backend mysql-primary
    balance roundrobin
    server primary 127.0.0.1:3306 check

backend mysql-replica
    balance leastconn
    server replica1 127.0.0.1:3307 check
    server replica2 127.0.0.1:3308 check
```

### 12.2 读写分离策略

| 策略 | 说明 | 配置 |
|------|------|------|
| Round-Robin | 轮询分配到从节点 | `balance roundrobin` |
| Least-Connections | 分配到连接数最少的从节点 | `balance leastconn` |
| Source | 同一客户端请求同一从节点 | `balance source` |

---

## 13. 安全配置

### 13.1 认证配置

```toml
[security]
enable_auth = true
default_auth_plugin = "mysql_native_password"
```

```sql
-- 设置 root 密码
ALTER USER 'root'@'localhost' IDENTIFIED BY 'your_password';

-- 创建应用用户
CREATE USER 'app'@'%' IDENTIFIED BY 'app_password';
GRANT ALL PRIVILEGES ON mydb.* TO 'app'@'%';

-- 列级权限 (T-17)
GRANT SELECT(email, phone) ON mydb.users TO 'reader'@'%';
REVOKE SELECT(salary) ON mydb.employees FROM 'reader'@'%';
```

### 13.2 SSL/TLS 配置

```bash
# 生成自签名证书
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
sudo mv cert.pem /etc/sqlrustgo/server.crt
sudo mv key.pem /etc/sqlrustgo/server.key
sudo chmod 600 /etc/sqlrustgo/server.key
```

```toml
[security]
tls_enabled = true
tls_cert = "/etc/sqlrustgo/server.crt"
tls_key = "/etc/sqlrustgo/server.key"
tls_min_version = "1.2"
```

```bash
# 使用 SSL 连接
mysql -h 127.0.0.1 -P 3306 -u root -p --ssl-ca=/etc/sqlrustgo/server.crt
```

### 13.3 防火墙配置

```bash
# 只允许应用服务器访问
ufw allow from 10.0.0.0/8 to any port 3306
ufw allow from 10.0.0.0/8 to any port 8080

# 拒绝外部直接访问
ufw deny 3306/tcp
```

---

## 14. 故障排查

### 14.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 连接拒绝 | 端口未开放 | 检查防火墙和绑定地址 |
| 内存不足 | buffer pool 过大 | 调整 `max_memory_mb` |
| 启动失败 | 数据目录权限 | `chmod 755 /var/lib/sqlrustgo` |
| 复制延迟 | 网络问题或从节点负载高 | 检查网络和从节点性能 |
| SIMD 未生效 | CPU 不支持 AVX2/AVX-512 | 自动 fallback 到标量 |

### 14.2 调试命令

```bash
# 检查服务状态
systemctl status sqlrustgo
docker ps | grep sqlrustgo

# 检查端口监听
netstat -an | grep 3306
ss -tlnp | grep 3306

# 检查日志
journalctl -u sqlrustgo -n 100 --no-pager
docker logs sqlrustgo --tail 100

# 测试连接
telnet 127.0.0.1 3306
curl http://127.0.0.1:8080/health

# SIMD 能力检测
cargo test -p sqlrustgo-vector -- detect_simd_lanes
```

### 14.3 日志位置

| 部署方式 | 日志位置 |
|----------|----------|
| systemd | `journalctl -u sqlrustgo` |
| Docker | `docker logs sqlrustgo` |
| 直接运行 | stdout |
| 文件 | `/var/log/sqlrustgo/` |

---

## 15. 升级

### 15.1 从 v2.7.0 升级

详见 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)

```bash
# 1. 备份数据
sqlrustgo backup --output /backup/v2.7.0-backup-$(date +%Y%m%d)

# 2. 停止服务
sudo systemctl stop sqlrustgo

# 3. 更新二进制
git fetch origin
git checkout develop/v2.8.0
cargo build --release --all-features
sudo cp target/release/sqlrustgo-mysql-server /usr/local/bin/

# 4. 启动服务
sudo systemctl start sqlrustgo

# 5. 验证
mysql -h 127.0.0.1 -P 3306 -u root -e "SELECT VERSION();"
```

### 15.2 回滚

```bash
# 1. 停止服务
sudo systemctl stop sqlrustgo

# 2. 恢复 v2.7.0 二进制
sudo cp /usr/local/bin/sqlrustgo-mysql-server.v2.7.0 /usr/local/bin/sqlrustgo-mysql-server

# 3. 恢复数据
sqlrustgo restore --input /backup/v2.7.0-backup-YYYYMMDD

# 4. 启动服务
sudo systemctl start sqlrustgo
```

---

## 相关文档

- [快速开始](./QUICK_START.md)
- [客户端连接](./CLIENT_CONNECTION.md)
- [迁移指南](./MIGRATION_GUIDE.md)
- [安全加固指南](./SECURITY_HARDENING.md)
- [REST API 参考](./API_REFERENCE.md)
- [发布门禁清单](./RELEASE_GATE_CHECKLIST.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-30*
