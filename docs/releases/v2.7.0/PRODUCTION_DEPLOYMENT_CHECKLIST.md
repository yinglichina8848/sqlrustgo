# v2.7.0 生产部署验收清单

> **版本**: v2.7.0 GA
> **日期**: 2026-04-22
> **状态**: ⏳ 待执行验收

---

## 1. 部署前检查

### 1.1 环境验证

| 检查项 | 验证命令 | 预期结果 | 状态 |
|--------|----------|----------|------|
| 硬件配置 | `nproc && free -h && df -h` | CPU≥4核, 内存≥8GB, 磁盘≥50GB | ⬜ |
| 操作系统 | `cat /etc/os-release` | Ubuntu 20.04+ / CentOS 8+ | ⬜ |
| Rust 环境 | `rustc --version && cargo --version` | Rust 1.85+ | ⬜ |
| 网络连通 | `curl -I https://github.com` | HTTP 200 | ⬜ |

### 1.2 二进制验证

| 检查项 | 验证命令 | 预期结果 | 状态 |
|--------|----------|----------|------|
| 二进制存在 | `ls -la /usr/local/bin/sqlrustgo` | 文件存在 | ⬜ |
| 版本正确 | `./sqlrustgo --version` | SQLRustGo v2.7.0 | ⬜ |
| 可执行权限 | `./sqlrustgo --help` | 帮助信息 | ⬜ |

---

## 2. 部署执行

### 2.1 系统用户创建

```bash
sudo useradd -r -s /bin/false sqlrustgo
sudo mkdir -p /var/lib/sqlrustgo
sudo mkdir -p /var/log/sqlrustgo
sudo chown -R sqlrustgo:sqlrustgo /var/lib/sqlrustgo
sudo chown -R sqlrustgo:sqlrustgo /var/log/sqlrustgo
```

### 2.2 配置文件创建

```bash
sudo mkdir -p /etc/sqlrustgo
sudo cat > /etc/sqlrustgo/config.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo"
wal_enabled = true

[logging]
level = "info"
log_dir = "/var/log/sqlrustgo"

[audit]
enabled = true
evidence_chain = true
EOF
sudo chown sqlrustgo:sqlrustgo /etc/sqlrustgo/config.toml
```

### 2.3 服务配置

```bash
sudo cat > /etc/systemd/system/sqlrustgo.service << 'EOF'
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
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable sqlrustgo
```

---

## 3. 部署后验证

### 3.1 服务状态检查

| 检查项 | 验证命令 | 预期结果 | 状态 |
|--------|----------|----------|------|
| 服务启动 | `sudo systemctl start sqlrustgo` | 无错误 | ⬜ |
| 服务状态 | `sudo systemctl status sqlrustgo` | active (running) | ⬜ |
| 进程存在 | `pgrep -f sqlrustgo` | PID 存在 | ⬜ |
| 端口监听 | `ss -tlnp \| grep 3306` | LISTEN | ⬜ |

### 3.2 基础功能验证

```bash
./sqlrustgo --execute "SELECT 1;"
./sqlrustgo --execute "CREATE TABLE deploy_test (id INTEGER, name TEXT);"
./sqlrustgo --execute "INSERT INTO deploy_test VALUES (1, 'deployment');"
./sqlrustgo --execute "SELECT * FROM deploy_test;"
```

### 3.3 数据完整性验证

```bash
./scripts/backup.sh -d default -o /tmp/deploy_test_backup -t full
./scripts/restore.sh -d default -b /tmp/deploy_test_backup -i /tmp --drop-first
```

---

## 4. 性能基线验证

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 简单查询 QPS | ≥1000 | TBD | ⬜ |
| 延迟 P50 | <10ms | TBD | ⬜ |
| 延迟 P99 | <50ms | TBD | ⬜ |

---

## 5. 验收签字

| 角色 | 姓名 | 日期 | 签字 |
|------|------|------|------|
| DevOps | | | |
| DBA | | | |
| 安全审核 | | | |
| 技术负责人 | | | |

**最终状态**: ⬜ 通过 / ❌ 失败

---

*文档由 OpenCode Agent 生成*
*最后更新: 2026-04-22*
