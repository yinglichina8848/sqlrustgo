# v3.1.0 安装指南

> **版本**: 3.1.0

---

## 系统要求

| 要求 | 最低配置 | 推荐配置 |
|------------|---------|-------------|
| CPU | 2 核 | 4+ 核 |
| 内存 | 4 GB | 8 GB |
| 磁盘 | 10 GB | 50 GB SSD |
| 操作系统 | macOS 12+, Linux (glibc 2.17+), Windows (WSL2) | macOS 14+, Ubuntu 22.04+ |
| Rust | 1.75+ | 1.80+ |

---

## 安装方式

### 1. 从源码构建（推荐）

```bash
# 克隆
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# 检出 v3.1.0
git checkout develop/v3.1.0

# 构建（包含所有特性）
cargo build --release --all-features

# 运行测试
cargo test --all-features
```

### 2. 二进制发布包（即将推出）

```bash
# 从 GitHub 下载（发布后可用）
curl -LO https://github.com/minzuuniversity/sqlrustgo/releases/download/v3.1.0/sqlrustgo-x86_64-apple-darwin.tar.gz
tar -xzf sqlrustgo-x86_64-apple-darwin.tar.gz
./sqlrustgo --version
```

### 3. Docker（即将推出）

```bash
# 拉取镜像
docker pull ghcr.io/minzuuniversity/sqlrustgo:v3.1.0

# 运行
docker run -p 3306:3306 \
  -v /data/sqlrustgo:/var/lib/sqlrustgo \
  ghcr.io/minzuuniversity/sqlrustgo:v3.1.0
```

---

## 初始配置

### 1. 初始化数据目录

```bash
# 创建数据目录
mkdir -p /var/lib/sqlrustgo
chmod 700 /var/lib/sqlrustgo

# 初始化（首次运行会创建系统表）
./target/release/sqlrustgo init --data-dir /var/lib/sqlrustgo
```

### 2. 配置

```bash
# 创建配置文件
cat > /etc/sqlrustgo.toml << 'EOF'
[data]
path = "/var/lib/sqlrustgo"

[server]
bind = "0.0.0.0:3306"
max_connections = 1000

[buffer_pool]
size = "128MB"

[wal]
enabled = true
sync = "O_DSYNC"

[security]
tls_enabled = true
tls_cert = "/etc/sqlrustgo/tls/server.crt"
tls_key = "/etc/sqlrustgo/tls/server.key"

[gmp]
audit_enabled = true
audit_chain_enabled = true
encryption_enabled = true
EOF
```

### 3. 启动服务

```bash
# 前台启动（测试用）
./target/release/sqlrustgo --config /etc/sqlrustgo.toml

# 守护进程模式启动
./target/release/sqlrustgo --config /etc/sqlrustgo.toml --daemon
```

### 4. 连接

```bash
# MySQL 客户端
mysql -h 127.0.0.1 -P 3306 -u root -p

# 或使用 TLS 连接
mysql -h 127.0.0.1 -P 3306 -u root -p --ssl-mode=REQUIRED
```

---

## 验证安装

```bash
# 检查版本
./target/release/sqlrustgo --version
# 输出: sqlrustgo 3.1.0

# 运行健康检查
./target/release/sqlrustgo health --data-dir /var/lib/sqlrustgo

# 连接并验证
mysql -h 127.0.0.1 -u root -e "SELECT VERSION();"
# 输出: 8.0.0-sqlrustgo-3.1.0
```

---

## GMP 安装（可选）

如需符合 GMP 标准的部署，请启用以下特性：

```bash
# 使用 GMP 特性构建
cargo build --release --all-features --features "gmp/audit,gmp/encryption"

# 配置 GMP
cat > /etc/sqlrustgo-gmp.toml << 'EOF'
[gmp]
audit_enabled = true
audit_chain_enabled = true
audit_sha256_required = true
encryption_enabled = true
encryption_algorithm = "AES-256-GCM"
gap_locking_enabled = true
serializable_enabled = true
EOF
```

---

## 卸载

```bash
# 停止服务
pkill -f sqlrustgo

# 删除数据（警告：此操作不可逆）
rm -rf /var/lib/sqlrustgo

# 删除二进制文件
rm /usr/local/bin/sqlrustgo

# 删除配置文件
rm /etc/sqlrustgo.toml
```

---

## 故障排除

| 问题 | 解决方案 |
|-------|----------|
| `error: cannot bind to port 3306` | 更改端口: `--server.bind=0.0.0.0:3307` |
| `error: data directory not initialized` | 运行: `./sqlrustgo init --data-dir /path` |
| `error: permission denied` | 检查目录权限: `chmod 700 /var/lib/sqlrustgo` |
| `error: TLS certificate not found` | 生成自签名证书: `scripts/generate_tls_cert.sh` |
