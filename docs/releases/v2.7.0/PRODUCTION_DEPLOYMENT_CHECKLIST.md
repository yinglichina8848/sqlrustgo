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
# 创建专用用户
sudo useradd -r -s /bin/false sqlrustgo

# 创建数据目录
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
# 连接测试
./sqlrustgo --execute "SELECT 1;"
# 预期: 显示查询结果

# 创建测试表
./sqlrustgo --execute "CREATE TABLE deploy_test (id INTEGER, name TEXT);"
./sqlrustgo --execute "INSERT INTO deploy_test VALUES (1, 'deployment');"
./sqlrustgo --execute "SELECT * FROM deploy_test;"
# 预期: 显示 1 | deployment

# WAL 功能验证
./sqlrustgo --execute "INSERT INTO deploy_test VALUES (2, 'wal_test');"
# 预期: 写入成功
```

### 3.3 数据完整性验证

```bash
# 备份测试
./scripts/backup.sh -d default -o /tmp/deploy_test_backup -t full
# 预期: 备份成功

# 恢复测试
./scripts/restore.sh -d default -b /tmp/deploy_test_backup -i /tmp --drop-first
# 预期: 恢复成功
```

---

## 4. 性能基线验证

### 4.1 基准 QPS

```bash
# 运行基准测试 (1分钟)
time ./target/release/sqlrustgo --execute "SELECT * FROM system.users;" --iterations 10000
```

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 简单查询 QPS | ≥1000 | TBD | ⬜ |
| 延迟 P50 | <10ms | TBD | ⬜ |
| 延迟 P99 | <50ms | TBD | ⬜ |

### 4.2 并发连接测试

```bash
# 并发测试 (10 连接)
for i in {1..10}; do
  ./sqlrustgo --execute "SELECT $i AS conn_id;" &
done
wait
```

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 10 并发 | 全部成功 | TBD | ⬜ |
| 50 并发 | 全部成功 | TBD | ⬜ |
| 100 并发 | 全部成功 | TBD | ⬜ |

---

## 5. 监控配置验证

### 5.1 日志检查

```bash
# 检查日志目录
ls -la /var/log/sqlrustgo/

# 检查错误日志
grep -i error /var/log/sqlrustgo/*.log
# 预期: 无 ERROR

# 检查审计日志
ls -la /var/log/sqlrustgo/audit/
```

### 5.2 资源监控

| 指标 | 监控命令 | 告警阈值 | 状态 |
|------|----------|----------|------|
| CPU 使用率 | `top -bn1 \| grep sqlrustgo` | <80% | ⬜ |
| 内存使用 | `ps aux \| grep sqlrustgo \| awk '{print $6}'` | <80% | ⬜ |
| 磁盘使用 | `df -h /var/lib/sqlrustgo` | <90% | ⬜ |

---

## 6. 安全验证

### 6.1 权限检查

```bash
# 检查文件权限
ls -la /var/lib/sqlrustgo/
# 预期: 仅 sqlrustgo 用户可读写

# 检查配置权限
ls -la /etc/sqlrustgo/config.toml
# 预期: 仅 root 可读，sqlrustgo 可读
```

### 6.2 网络访问控制

| 检查项 | 验证方法 | 状态 |
|--------|----------|------|
| 本地访问 | `mysql -h 127.0.0.1 -P 3306` | ⬜ |
| 远程拒绝 | `mysql -h <public_ip> -P 3306` | ⬜ (应拒绝) |

---

## 7. 验收签字

| 角色 | 姓名 | 日期 | 签字 |
|------|------|------|------|
| DevOps | | | |
| DBA | | | |
| 安全审核 | | | |
| 技术负责人 | | | |

---

## 8. 回滚预案

如部署失败，执行以下回滚：

```bash
# 1. 停止服务
sudo systemctl stop sqlrustgo

# 2. 恢复备份
./scripts/restore.sh -d default -b <backup_path> -i /backups --drop-first

# 3. 验证数据
./sqlrustgo --execute "SELECT COUNT(*) FROM your_tables;"

# 4. 重启旧版本服务（如有）
sudo systemctl restart sqlrustgo
```

---

## 9. 部署完成确认

| 门禁项 | 状态 | 说明 |
|--------|------|------|
| 环境验证 | ⬜ | |
| 二进制部署 | ⬜ | |
| 配置完成 | ⬜ | |
| 服务启动 | ⬜ | |
| 基础功能 | ⬜ | |
| 数据完整性 | ⬜ | |
| 性能基线 | ⬜ | |
| 监控配置 | ⬜ | |
| 安全验证 | ⬜ | |
| 验收签字 | ⬜ | |

**最终状态**: ⬜ 通过 / ❌ 失败

---

*文档由 OpenCode Agent 生成*
*最后更新: 2026-04-22*
