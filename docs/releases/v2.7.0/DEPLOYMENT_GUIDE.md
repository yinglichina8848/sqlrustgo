# SQLRustGo v2.7.0 部署指南

> **版本**: v2.7.0
> **最后更新**: 2026-04-22

---

## 概述

本文档提供 SQLRustGo v2.7.0 的部署指南，包括环境要求、配置选项和部署步骤。

---

## 1. 环境要求

### 1.1 硬件要求

| 组件 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 4+ 核 |
| 内存 | 4 GB | 8+ GB |
| 磁盘 | 10 GB SSD | 50+ GB SSD |

### 1.2 软件要求

- **操作系统**: Linux (Ubuntu 20.04+, CentOS 8+), macOS 12+, Windows 10+ (WSL2)
- **Rust**: 1.85.0+
- **Cargo**: 最新稳定版
- **依赖**: OpenSSL (Linux), CMake 3.15+ (编译依赖)

---

## 2. Linux/macOS 部署

### 2.1 从源码编译

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.7.0 分支
git checkout develop/v2.7.0

# 编译发布版本
cargo build --release --all-features

# 安装
cargo install --path .
```

### 2.2 使用预编译二进制

从 [Releases](https://github.com/minzuuniversity/sqlrustgo/releases) 页面下载对应平台的二进制文件。

```bash
# Linux/macOS
chmod +x sqlrustgo-*-unknown-linux-gnu
sudo mv sqlrustgo-*-unknown-linux-gnu /usr/local/bin/sqlrustgo

# 验证安装
sqlrustgo --version
```

### 2.3 Linux 系统服务 (systemd)

```bash
# 创建系统用户
sudo useradd -r -s /bin/false sqlrustgo

# 创建服务文件 /etc/systemd/system/sqlrustgo.service
[Unit]
Description=SQLRustGo Database v2.7.0
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo server --config /etc/sqlrustgo/config.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
# 启用并启动服务
sudo systemctl enable sqlrustgo
sudo systemctl start sqlrustgo

# 查看状态
sudo systemctl status sqlrustgo
```

### 2.4 macOS Launchd

创建 `~/Library/LaunchAgents/com.sqlrustgo.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.sqlrustgo.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/sqlrustgo</string>
        <string>server</string>
        <string>--config</string>
        <string>/etc/sqlrustgo/config.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

```bash
# 加载服务
launchctl load ~/Library/LaunchAgents/com.sqlrustgo.server.plist
```

---

## 3. Docker 部署

### 3.1 使用官方镜像

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v2.7.0

# 运行容器
docker run -d \
  --name sqlrustgo \
  -p 3306:3306 \
  -v sqlrustgo-data:/data \
  minzuuniversity/sqlrustgo:v2.7.0
```

### 3.2 Docker Compose 部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  sqlrustgo:
    image: minzuuniversity/sqlrustgo:v2.7.0
    container_name: sqlrustgo
    ports:
      - "3306:3306"
    volumes:
      - sqlrustgo-data:/data
    environment:
      - SQLRUSTGO_MODE=production
      - SQLRUSTGO_MAX_CONNECTIONS=100
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "sqlrustgo", "health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  sqlrustgo-data:
```

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f sqlrustgo
```

### 3.3 Docker 资源限制

```yaml
services:
  sqlrustgo:
    image: minzuuniversity/sqlrustgo:v2.7.0
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

---

## 4. Kubernetes 部署

### 4.1 Deployment

```yaml
# sqlrustgo-deployment.yaml
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
        image: minzuuniversity/sqlrustgo:v2.7.0
        ports:
        - containerPort: 3306
          name: mysql
        - containerPort: 9090
          name: metrics
        env:
        - name: SQLRUSTGO_MODE
          value: "production"
        - name: SQLRUSTGO_MAX_CONNECTIONS
          value: "100"
        resources:
          requests:
            memory: "2Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          tcpSocket:
            port: 3306
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command: ["sqlrustgo", "health"]
          initialDelaySeconds: 5
          periodSeconds: 10
```

### 4.2 Service

```yaml
# sqlrustgo-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: sqlrustgo-service
spec:
  type: ClusterIP
  selector:
    app: sqlrustgo
  ports:
  - name: mysql
    port: 3306
    targetPort: 3306
  - name: metrics
    port: 9090
    targetPort: 9090
```

### 4.3 PersistentVolumeClaim

```yaml
# sqlrustgo-pvc.yaml
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

### 4.4 部署命令

```bash
# 部署
kubectl apply -f sqlrustgo-pvc.yaml
kubectl apply -f sqlrustgo-deployment.yaml
kubectl apply -f sqlrustgo-service.yaml

# 查看状态
kubectl get pods -l app=sqlrustgo
kubectl get svc sqlrustgo-service

# 查看日志
kubectl logs -l app=sqlrustgo -f
```

---

## 5. 云平台部署

### 5.1 AWS (EC2/ECS)

#### EC2 部署

```bash
# 使用 AWS CLI 启动实例
aws ec2 run-instances \
  --image-id ami-0c55b159cbfafe1f0 \
  --instance-type t3.medium \
  --key-name your-key-pair \
  --security-group-ids sg-xxxxxxxx \
  --subnet-id subnet-xxxxxxxx

# 在实例上安装
sudo yum install -y docker
sudo systemctl start docker
docker run -d --name sqlrustgo \
  -p 3306:3306 \
  -v sqlrustgo-data:/data \
  minzuuniversity/sqlrustgo:v2.7.0
```

#### ECS 任务定义

```json
{
  "family": "sqlrustgo",
  "containerDefinitions": [
    {
      "name": "sqlrustgo",
      "image": "minzuuniversity/sqlrustgo:v2.7.0",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 3306,
          "hostPort": 3306
        }
      ],
      "environment": [
        {"name": "SQLRUSTGO_MODE", "value": "production"}
      ],
      "logConfiguration": {
        "logDriver": "awslogs"
      }
    }
  ]
}
```

### 5.2 Google Cloud Platform (GCE/GKE)

#### GKE 部署

```bash
# 创建集群
gcloud container clusters create sqlrustgo-cluster \
  --zone us-central1-a \
  --num-nodes=2

# 获取凭证
gcloud container clusters get-credentials sqlrustgo-cluster

# 部署
kubectl apply -f sqlrustgo-deployment.yaml
kubectl apply -f sqlrustgo-service.yaml
```

#### Cloud SQL 替代方案

对于生产环境，建议使用 Cloud SQL 作为托管数据库服务:

```bash
# 创建 Cloud SQL 实例
gcloud sql instances create sqlrustgo \
  --database-version=MYSQL_8_0 \
  --tier=db-n1-standard-2 \
  --region=us-central1
```

### 5.3 Microsoft Azure (VM/AKS)

#### AKS 部署

```bash
# 创建资源组
az group create --name sqlrustgo-rg --location eastus

# 创建 AKS 集群
az aks create \
  --resource-group sqlrustgo-rg \
  --name sqlrustgo-cluster \
  --node-count 2 \
  --enable-addons monitoring

# 获取凭证
az aks get-credentials --resource-group sqlrustgo-rg --name sqlrustgo-cluster

# 部署
kubectl apply -f sqlrustgo-deployment.yaml
```

#### Azure Database for MySQL 替代方案

对于生产环境，建议使用 Azure Database for MySQL:

```bash
# 创建服务器
az mysql server create \
  --resource-group sqlrustgo-rg \
  --name sqlrustgomyserver \
  --admin-user sqlrustgoadmin \
  --admin-password YourPassword! \
  --sku-name B_Gen5_2
```

### 5.4 阿里云 (ECS/ACK)

```bash
# 创建 ACK 集群
aliyun cs POST /clusters \
  --body '{
    "name": "sqlrustgo-cluster",
    "node_count": 2,
    "region": "cn-beijing"
  }'

# 部署应用
kubectl apply -f sqlrustgo-deployment.yaml
```

---

## 6. 配置

### 6.1 配置文件

默认配置文件路径: `/etc/sqlrustgo/config.toml` 或 `./config.toml`

```toml
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo"
temp_dir = "/tmp/sqlrustgo"

[storage.engine]
type = "buffer_pool"
max_memory_mb = 2048

[logging]
level = "info"
output = "stdout"

[security]
enable_auth = true
```

### 6.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_HOST` | 监听地址 | 0.0.0.0 |
| `SQLRUSTGO_PORT` | 监听端口 | 3306 |
| `SQLRUSTGO_DATA_DIR` | 数据目录 | /data |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 | info |
| `SQLRUSTGO_MAX_CONNECTIONS` | 最大连接数 | 100 |

---

## 7. 初始化

### 7.1 初始化数据目录

```bash
# 初始化存储
sqlrustgo init --data-dir /var/lib/sqlrustgo
```

### 7.2 创建数据库

```sql
CREATE DATABASE IF NOT EXISTS test_db;
USE test_db;

-- 创建示例表
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100)
);
```

---

## 8. 高可用部署

### 8.1 主从复制

```toml
[replication]
enabled = true
role = "primary"
sync_mode = "semi-sync"
```

从节点配置:

```toml
[replication]
enabled = true
role = "replica"
primary_address = "192.168.1.1:3306"
```

### 8.2 连接池

建议使用连接池中间件 (如 ProxySQL) 或应用层连接池。

---

## 9. 监控

### 9.1 健康检查

```bash
# HTTP 健康检查
curl http://localhost:3306/health

# MySQL 协议 ping
mysqladmin ping -h localhost -P 3306
```

### 9.2 指标导出

支持 Prometheus 格式指标:

```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"
```

---

## 10. 备份与恢复

### 10.1 备份

```bash
# 全量备份
sqlrustgo backup --output /backup/sqlrustgo-$(date +%Y%m%d).dump
```

### 10.2 恢复

```bash
# 恢复数据
sqlrustgo restore --input /backup/sqlrustgo-20260422.dump
```

---

## 11. 安全

### 11.1 防火墙配置

```bash
# 只允许应用服务器访问
ufw allow from 10.0.0.0/8 to any port 3306
```

### 11.2 SSL/TLS

```toml
[security]
tls_enabled = true
tls_cert = "/etc/sqlrustgo/server.crt"
tls_key = "/etc/sqlrustgo/server.key"
```

---

## 12. 故障排查

### 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 连接拒绝 | 端口未开放 | 检查防火墙和绑定地址 |
| 内存不足 | buffer pool 过大 | 调整 max_memory_mb |
| 启动失败 | 数据目录权限 | chmod 755 data_dir |

### 日志位置

- 默认: stdout (Docker)
- 文件: `/var/log/sqlrustgo/`

---

## 相关文档

- [用户手册](oo/user-guide/USER_MANUAL.md)
- [升级指南](UPGRADE_GUIDE.md)
- [性能目标](PERFORMANCE_TARGETS.md)

---

*本文档由 SQLRustGo Team 维护*
