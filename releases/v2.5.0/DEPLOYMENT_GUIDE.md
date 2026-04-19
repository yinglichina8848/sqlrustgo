# SQLRustGo v2.5.0 部署指南

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16

---

## 一、系统概述

### 1.1 组件架构

```
┌─────────────────────────────────────────────────────────────┐
│                    SQLRustGo v2.5.0                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │   CLI   │  │ Server  │  │ Bench   │  │ Tools   │        │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Core Crates                             │    │
│  │  parser | planner | executor | optimizer | storage   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 系统要求

#### 硬件要求

| 组件 | 最低要求 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 8 核+ |
| 内存 | 4 GB | 32 GB+ |
| 磁盘 | 10 GB | 100 GB+ SSD |
| 网络 | 100 Mbps | 1 Gbps |

#### 软件要求

| 软件 | 版本要求 |
|------|----------|
| Rust | 1.70+ (2021 edition) |
| 操作系统 | Linux (Ubuntu 20.04+), macOS 12+, Windows 10+ |

---

## 二、二进制安装

### 2.1 从源码编译

```bash
# 1. 克隆代码
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 2. 切换到 v2.5.0
git checkout develop/v2.5.0

# 3. 编译
cargo build --release

# 4. 安装
cargo install --path .
```

### 2.2 验证安装

```bash
# 检查版本
sqlrustgo --version
# 输出: sqlrustgo 2.5.0

# 运行帮助
sqlrustgo --help
```

---

## 三、Docker 部署

### 3.1 使用预构建镜像

```bash
# 拉取镜像
docker pull sqlrustgo/sqlrustgo:2.5.0

# 运行容器
docker run -d \
  --name sqlrustgo \
  -p 5432:5432 \
  -v /data/sqlrustgo:/var/lib/sqlrustgo \
  sqlrustgo/sqlrustgo:2.5.0
```

### 3.2 Docker Compose 部署

创建 `docker-compose.yml`:

```yaml
version: '3.8'

services:
  sqlrustgo:
    image: sqlrustgo/sqlrustgo:2.5.0
    container_name: sqlrustgo
    ports:
      - "5432:5432"
      - "8080:8080"
    volumes:
      - sqlrustgo_data:/var/lib/sqlrustgo
      - ./config.toml:/etc/sqlrustgo.toml:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  sqlrustgo_data:
```

启动服务:

```bash
docker-compose up -d

# 查看日志
docker-compose logs -f sqlrustgo

# 检查状态
docker-compose ps
```

### 3.3 自定义 Docker 镜像

创建 `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sqlrustgo-server /usr/local/bin/
COPY --from=builder /app/target/release/sqlrustgo /usr/local/bin/

EXPOSE 5432 8080

ENTRYPOINT ["sqlrustgo-server"]
CMD ["--config", "/etc/sqlrustgo.toml"]
```

构建并部署:

```bash
# 构建镜像
docker build -t my-sqlrustgo:2.5.0 .

# 运行
docker run -d -p 5432:5432 my-sqlrustgo:2.5.0
```

---

## 四、Kubernetes 部署

### 4.1 创建配置

创建 `sqlrustgo-config.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: sqlrustgo-config
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 5432
    max_connections = 100

    [storage]
    data_dir = "/var/lib/sqlrustgo/data"
    wal_dir = "/var/lib/sqlrustgo/wal"

    [mvcc]
    enabled = true

    [wal]
    enabled = true
```

### 4.2 创建 PVC

创建 `sqlrustgo-pvc.yaml`:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: sqlrustgo-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
```

### 4.3 创建 Deployment

创建 `sqlrustgo-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sqlrustgo
  labels:
    app: sqlrustgo
spec:
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
          image: sqlrustgo/sqlrustgo:2.5.0
          ports:
            - containerPort: 5432
              name: sql
            - containerPort: 8080
              name: http
          volumeMounts:
            - name: data
              mountPath: /var/lib/sqlrustgo
            - name: config
              mountPath: /etc/sqlrustgo.toml
              subPath: config.toml
          resources:
            requests:
              memory: "2Gi"
              cpu: "1000m"
            limits:
              memory: "8Gi"
              cpu: "4000m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: sqlrustgo-data
        - name: config
          configMap:
            name: sqlrustgo-config
```

### 4.4 创建 Service

创建 `sqlrustgo-service.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-service
spec:
  type: LoadBalancer
  ports:
    - port: 5432
      targetPort: 5432
      protocol: TCP
      name: sql
    - port: 8080
      targetPort: 8080
      protocol: TCP
      name: http
  selector:
    app: sqlrustgo
```

### 4.5 部署

```bash
# 部署配置
kubectl apply -f sqlrustgo-config.yaml
kubectl apply -f sqlrustgo-pvc.yaml
kubectl apply -f sqlrustgo-deployment.yaml
kubectl apply -f sqlrustgo-service.yaml

# 查看部署状态
kubectl get pods -l app=sqlrustgo
kubectl get svc sqlrustgo-service

# 查看日志
kubectl logs -l app=sqlrustgo -f
```

---

## 五、生产环境配置

### 5.1 完整配置文件

```toml
# /etc/sqlrustgo.toml

[server]
host = "0.0.0.0"
port = 5432
max_connections = 100
connection_timeout_seconds = 30
idle_timeout_seconds = 300

[storage]
data_dir = "/var/lib/sqlrustgo/data"
wal_dir = "/var/lib/sqlrustgo/wal"
temp_dir = "/var/lib/sqlrustgo/temp"

[mvcc]
enabled = true
snapshot_isolation = true
gc_interval_seconds = 300
max_versions = 1000

[wal]
enabled = true
wal_file_size_mb = 256
sync_mode = "fsync"  # fsync, fdatasync, none
pitr_enabled = true
archive_enabled = true
archive_retention_days = 7

[vector_index]
default_type = "hnsw"
hnsw_m = 16
hnsw_ef_construction = 200
hnsw_ef_search = 100

[graph]
enabled = true
graph_dir = "/var/lib/sqlrustgo/graph"
cypher_enabled = true

[unified_query]
enabled = true
fusion_mode = "rrf"

[performance]
simd_enabled = true
parallel_execution = true
max_thread_count = 0
batch_size = 1024

[logging]
level = "info"
path = "/var/log/sqlrustgo/"
max_file_size_mb = 100
max_backup_files = 10

[security]
tls_enabled = false
tls_cert_path = "/etc/sqlrustgo/tls.crt"
tls_key_path = "/etc/sqlrustgo/tls.key"

[backup]
enabled = true
backup_dir = "/var/backup/sqlrustgo"
schedule = "0 2 * * *"  # 每天凌晨2点
retention_days = 30
```

### 5.2 系统服务配置

创建 systemd 服务文件 `/etc/systemd/system/sqlrustgo.service`:

```ini
[Unit]
Description=SQLRustGo v2.5.0
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
WorkingDirectory=/var/lib/sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo-server --config /etc/sqlrustgo.toml
Restart=always
RestartSec=10
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

启用服务:

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务
sudo systemctl enable sqlrustgo

# 启动服务
sudo systemctl start sqlrustgo

# 查看状态
sudo systemctl status sqlrustgo

# 查看日志
sudo journalctl -u sqlrustgo -f
```

---

## 六、监控与运维

### 6.1 健康检查

```bash
# 健康检查
curl http://localhost:8080/health
# 输出: {"status":"ok"}

# 就绪检查
curl http://localhost:8080/ready
# 输出: {"status":"ready"}
```

### 6.2 Prometheus 指标

启用 Prometheus 端点:

```toml
[observability]
prometheus_enabled = true
prometheus_port = 9090
```

获取指标:

```bash
curl http://localhost:9090/metrics
```

### 6.3 日志管理

```bash
# 查看日志
tail -f /var/log/sqlrustgo/sqlrustgo.log

# 日志轮转配置 /etc/logrotate.d/sqlrustgo
/var/log/sqlrustgo/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 sqlrustgo sqlrustgo
}
```

### 6.4 备份操作

```bash
# 手动备份
sqlrustgo-tools backup --dir /var/backup/sqlrustgo/$(date +%Y%m%d)

# 列出备份
sqlrustgo-tools backup-list --dir /var/backup/sqlrustgo

# 恢复备份
sqlrustgo-tools restore --dir /var/backup/sqlrustgo/20260416
```

---

## 七、故障排查

### 7.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 服务启动失败 | 端口被占用 | 检查端口占用 `netstat -tlnp \| grep 5432` |
| 连接超时 | 防火墙阻止 | 开放端口 `ufw allow 5432` |
| 性能下降 | 内存不足 | 增加内存或调整 `max_thread_count` |
| 索引损坏 | 异常关机 | 运行 `sqlrustgo-tools rebuild-index` |

### 7.2 日志分析

```bash
# 查看错误日志
grep -i error /var/log/sqlrustgo/sqlrustgo.log

# 查看警告
grep -i warn /var/log/sqlrustgo/sqlrustgo.log

# 查看慢查询
grep -i "slow query" /var/log/sqlrustgo/sqlrustgo.log
```

### 7.3 性能诊断

```bash
# 检查当前连接
curl http://localhost:8080/api/connections

# 检查活跃事务
curl http://localhost:8080/api/transactions

# 检查缓存命中率
curl http://localhost:8080/api/cache/stats
```

---

## 八、安全配置

### 8.1 TLS 配置

```toml
[security]
tls_enabled = true
tls_cert_path = "/etc/sqlrustgo/tls.crt"
tls_key_path = "/etc/sqlrustgo/tls.key"
require_ssl = true
```

### 8.2 防火墙配置

```bash
# Ubuntu/Debian
sudo ufw allow 5432/tcp
sudo ufw allow 8080/tcp
sudo ufw enable

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=5432/tcp
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload
```

---

## 九、扩展阅读

| 文档 | 说明 |
|------|------|
| [USER_MANUAL.md](./oo/user-guide/USER_MANUAL.md) | 用户手册 |
| [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) | 升级指南 |
| [SECURITY_ANALYSIS.md](./oo/reports/SECURITY_ANALYSIS.md) | 安全分析 |

---

*部署指南 v2.5.0*
*最后更新: 2026-04-16*
