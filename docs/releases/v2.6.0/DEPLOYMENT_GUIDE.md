# SQLRustGo v2.6.0 部署指南

> **版本**: v2.6.0
> **最后更新**: 2026-04-18

---

## 概述

本文档提供 SQLRustGo v2.6.0 的部署指南，包括环境要求、配置选项和部署步骤。

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

## 2. 二进制部署

### 2.1 从源码编译

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.6.0 分支
git checkout develop/v2.6.0

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

---

## 3. Docker 部署

### 3.1 使用官方镜像

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v2.6.0

# 运行容器
docker run -d \
  --name sqlrustgo \
  -p 3306:3306 \
  -v sqlrustgo-data:/data \
  minzuuniversity/sqlrustgo:v2.6.0
```

### 3.2 Docker Compose 部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  sqlrustgo:
    image: minzuuniversity/sqlrustgo:v2.6.0
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

---

## 4. 配置

### 4.1 配置文件

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

### 4.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_HOST` | 监听地址 | 0.0.0.0 |
| `SQLRUSTGO_PORT` | 监听端口 | 3306 |
| `SQLRUSTGO_DATA_DIR` | 数据目录 | /data |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 | info |
| `SQLRUSTGO_MAX_CONNECTIONS` | 最大连接数 | 100 |

---

## 5. 初始化

### 5.1 初始化数据目录

```bash
# 初始化存储
sqlrustgo init --data-dir /var/lib/sqlrustgo
```

### 5.2 创建数据库

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

## 6. 高可用部署

### 6.1 主从复制

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

### 6.2 连接池

建议使用连接池中间件 (如 ProxySQL) 或应用层连接池。

---

## 7. 监控

### 7.1 健康检查

```bash
# HTTP 健康检查
curl http://localhost:3306/health

# MySQL 协议 ping
mysqladmin ping -h localhost -P 3306
```

### 7.2 指标导出

支持 Prometheus 格式指标:

```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"
```

---

## 8. 备份与恢复

### 8.1 备份

```bash
# 全量备份
sqlrustgo backup --output /backup/sqlrustgo-$(date +%Y%m%d).dump
```

### 8.2 恢复

```bash
# 恢复数据
sqlrustgo restore --input /backup/sqlrustgo-20260418.dump
```

---

## 9. 安全

### 9.1 防火墙配置

```bash
# 只允许应用服务器访问
ufw allow from 10.0.0.0/8 to any port 3306
```

### 9.2 SSL/TLS

```toml
[security]
tls_enabled = true
tls_cert = "/etc/sqlrustgo/server.crt"
tls_key = "/etc/sqlrustgo/server.key"
```

---

## 10. 故障排查

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

- [用户手册](USER_MANUAL.md)
- [升级指南](UPGRADE_GUIDE.md)
- [性能目标](PERFORMANCE_TARGETS.md)

---

*本文档由 SQLRustGo Team 维护*
