# v2.8.0 生产部署验收清单

> **版本**: v2.8.0 GA
> **日期**: 2026-05-02
> **基于**: v2.7.0 基线与 v2.8.0 新增分布式/安全能力

---

## 1. 部署前检查

### 1.1 环境验证

| 检查项 | 验证命令 | 预期结果 | 状态 |
|--------|----------|----------|------|
| 硬件配置 | `nproc && free -h && df -h` | CPU≥4核, 内存≥8GB, 磁盘≥50GB | ⬜ |
| 操作系统 | `cat /etc/os-release` | Ubuntu 20.04+ / CentOS 8+ / macOS 14+ | ⬜ |
| Rust 环境 | `rustc --version && cargo --version` | Rust 1.70+ | ⬜ |
| 网络连通 | `ping -c 3 192.168.0.252` | 0% packet loss | ⬜ |
| SSH 免密 | `ssh -T gitea-devstack` | 认证成功 | ⬜ |
| 时钟同步 | `timedatectl status` | NTP 同步, 偏差 < 1s | ⬜ |

### 1.2 二进制验证

| 检查项 | 验证命令 | 预期结果 | 状态 |
|--------|----------|----------|------|
| 版本正确 | `./target/release/sqlrustgo --version` | SQLRustGo v2.8.0 | ⬜ |
| 帮助信息 | `./target/release/sqlrustgo --help` | 显示帮助信息 | ⬜ |
| CLI 工具 | `./target/release/sqlrustgo-tools --help` | 显示工具帮助 | ⬜ |
| DISTRIBUTED 模式 | `./target/release/sqlrustgo --mode distributed --help` | 分布式参数可用 | ⬜ |

---

## 2. 基础部署

### 2.1 系统用户创建

```bash
sudo useradd -r -s /bin/false sqlrustgo
sudo mkdir -p /var/lib/sqlrustgo
sudo mkdir -p /var/log/sqlrustgo
sudo mkdir -p /var/lib/sqlrustgo/backups
sudo mkdir -p /var/lib/sqlrustgo/wal
sudo chown -R sqlrustgo:sqlrustgo /var/lib/sqlrustgo
sudo chown -R sqlrustgo:sqlrustgo /var/log/sqlrustgo
```

### 2.2 二进制部署

```bash
# 编译
cargo build --release --all-features

# 安装到系统路径
sudo cp target/release/sqlrustgo /usr/local/bin/
sudo cp target/release/sqlrustgo-tools /usr/local/bin/
sudo chmod +x /usr/local/bin/sqlrustgo /usr/local/bin/sqlrustgo-tools

# 验证安装
sqlrustgo --version
sqlrustgo-tools --version
```

### 2.3 配置文件

创建 `/etc/sqlrustgo/config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo/data"
wal_dir = "/var/lib/sqlrustgo/wal"
buffer_pool_size_mb = 1024

[security]
# TLS 配置
tls_enabled = false
tls_cert_path = "/etc/sqlrustgo/certs/server.crt"
tls_key_path = "/etc/sqlrustgo/certs/server.key"

# SQL 防火墙
firewall_enabled = true
query_timeout_secs = 60
max_rows = 10000

# 审计日志
audit_enabled = true
audit_log_path = "/var/log/sqlrustgo/audit.log"

[backup]
backup_dir = "/var/lib/sqlrustgo/backups"
schedule = "daily"
retention_days = 7
compress = true

[distributed]
mode = "standalone"
# 集群模式:
# mode = "cluster"
# node_id = 1
# seed_nodes = ["192.168.1.10:3306", "192.168.1.11:3306"]

[logging]
level = "info"
log_dir = "/var/log/sqlrustgo"
```

---

## 3. 安全加固

| 检查项 | 操作 | 验证方式 | 状态 |
|--------|------|----------|------|
| 文件权限 | `chmod 600 /etc/sqlrustgo/config.toml` | `ls -la /etc/sqlrustgo/` | ⬜ |
| 审计日志 | `chmod 640 /var/log/sqlrustgo/audit.log` | `ls -la /var/log/sqlrustgo/` | ⬜ |
| WAL 目录 | `chmod 700 /var/lib/sqlrustgo/wal` | `ls -la /var/lib/sqlrustgo/wal` | ⬜ |
| 防火墙规则 | `sudo ufw allow 3306/tcp` | `sudo ufw status` | ⬜ |
| 进程权限 | `sudo -u sqlrustgo ./sqlrustgo` | 以 sqlrustgo 用户运行 | ⬜ |
| SQL 防火墙 | config.toml 中 `firewall_enabled = true` | 验证防火墙拦截 | ⬜ |
| TLS 证书 | 配置 cert/key 路径 | `openssl x509 -in /etc/sqlrustgo/certs/server.crt -text` | ⬜ |

---

## 4. 功能验证

### 4.1 基本功能

```bash
# 启动服务
sqlrustgo --config /etc/sqlrustgo/config.toml &

# 连接测试
mysql -h 127.0.0.1 -P 3306 -u root -e "SELECT VERSION();"

# DDL 测试
mysql -h 127.0.0.1 -P 3306 -u root -e "CREATE DATABASE test;"
mysql -h 127.0.0.1 -P 3306 -u root -e "CREATE TABLE test.users (id INT PRIMARY KEY, name TEXT);"

# DML 测试
mysql -h 127.0.0.1 -P 3306 -u root -e "INSERT INTO test.users VALUES (1, 'Alice');"
mysql -h 127.0.0.1 -P 3306 -u root -e "SELECT * FROM test.users;"
```

### 4.2 v2.8.0 新增功能验证

```bash
# 分区表
mysql -h 127.0.0.1 -P 3306 -u root \
  -e "CREATE TABLE test.orders (id INT, ts TIMESTAMP) PARTITION BY RANGE (id) (PARTITION p0 VALUES LESS THAN (100), PARTITION p1 VALUES LESS THAN (200));"

# FULL OUTER JOIN
mysql -h 127.0.0.1 -P 3306 -u root \
  -e "SELECT * FROM test.users u FULL OUTER JOIN test.orders o ON u.id = o.user_id;"

# TRUNCATE
mysql -h 127.0.0.1 -P 3306 -u root -e "TRUNCATE TABLE test.users;"

# REPLACE INTO
mysql -h 127.0.0.1 -P 3306 -u root -e "REPLACE INTO test.users VALUES (1, 'Bob');"

# 窗口函数
mysql -h 127.0.0.1 -P 3306 -u root \
  -e "SELECT id, name, ROW_NUMBER() OVER (ORDER BY id) as rn FROM test.users;"

# CHECK 约束
mysql -h 127.0.0.1 -P 3306 -u root \
  -e "CREATE TABLE test.accounts (id INT, balance INT CHECK (balance >= 0));"
```

### 4.3 安全功能验证

```bash
# SQL 注入检测
mysql -h 127.0.0.1 -P 3306 -u root -e "SELECT * FROM users WHERE id = 1 OR 1=1;"
# 预期: 被防火墙拦截

# 查询超时
mysql -h 127.0.0.1 -P 3306 -u root -e "SELECT SLEEP(100);"
# 预期: 60s 后超时断开

# 审计日志检查
cat /var/log/sqlrustgo/audit.log | tail -10
# 预期: 所有 SQL 操作被记录
```

### 4.4 备份恢复验证

```bash
# 全量备份
sqlrustgo-tools backup --database test --output-dir /var/lib/sqlrustgo/backups --backup-type full

# 验证备份文件
ls -la /var/lib/sqlrustgo/backups/

# 恢复
sqlrustgo-tools restore --database test --backup-id <id> --backup-dir /var/lib/sqlrustgo/backups
```

---

## 5. 性能验证

| 测试项 | 验证命令 | 最低要求 | 状态 |
|--------|----------|----------|------|
| QPS 基本 | `cargo test --test qps_benchmark_test -- --nocapture` | ≥ 1000 QPS | ⬜ |
| 并发连接 | `sysbench oltp_read_write --threads=50 run` | 50 并发稳定 | ⬜ |
| 内存使用 | `ps -eo pid,rss,command | grep sqlrustgo` | < 8GB RSS | ⬜ |
| SIMD 加速 | `cargo test -p sqlrustgo-vector -- simd` | All tests PASS | ⬜ |

---

## 6. 监控配置

### 6.1 日志监控

```bash
# 错误日志
tail -f /var/log/sqlrustgo/error.log

# 慢查询日志（如果启用）
tail -f /var/log/sqlrustgo/slow_query.log

# 审计日志
tail -f /var/log/sqlrustgo/audit.log
```

### 6.2 进程监控 (systemd)

创建 `/etc/systemd/system/sqlrustgo.service`:

```ini
[Unit]
Description=SQLRustGo Database Server
After=network.target

[Service]
Type=simple
User=sqlrustgo
Group=sqlrustgo
ExecStart=/usr/local/bin/sqlrustgo --config /etc/sqlrustgo/config.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable sqlrustgo
sudo systemctl start sqlrustgo
sudo systemctl status sqlrustgo
```

---

## 7. 验收总表

| 验收类别 | 检查项数 | 通过 | 失败 | 完成度 |
|----------|----------|------|------|--------|
| 环境验证 | 6 | ⬜ | ⬜ | 0% |
| 基础部署 | 4 | ⬜ | ⬜ | 0% |
| 安全加固 | 8 | ⬜ | ⬜ | 0% |
| 功能验证 | 12 | ⬜ | ⬜ | 0% |
| 性能验证 | 4 | ⬜ | ⬜ | 0% |
| 监控配置 | 3 | ⬜ | ⬜ | 0% |
| **总计** | **37** | **0** | **0** | **0%** |

> 注意: 部署前请先阅读 `docs/releases/v2.8.0/UPGRADE_GUIDE.md` 了解 Breaking Changes 和迁移步骤。
> 安全配置详见 `docs/releases/v2.8.0/SECURITY_HARDENING.md`。
