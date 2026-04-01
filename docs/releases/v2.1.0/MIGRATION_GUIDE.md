# SQLRustGo v2.1.0 从 v2.0 升级指南

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、升级概述

v2.1.0 是 v2.0 的增量升级版本，主要新增功能包括：
- 可观测性增强 (Prometheus/Grafana/慢查询)
- SQL Firewall 安全增强
- 备份工具完善 (Physical Backup/保留策略)
- 新数据类型 (UUID/ARRAY/ENUM)
- TPC-H SQL 功能增强

### 兼容性

| 组件 | 兼容性 |
|------|---------|
| SQL 语法 | ✅ 兼容 |
| 配置文件 | ✅ 兼容 |
| 存储格式 | ✅ 兼容 |
| API 接口 | ✅ 兼容 |

---

## 二、升级步骤

### 2.1 备份数据

```bash
# 备份数据库
sqlrustgo-tools mysqldump --database mydb --out /backup/mydb.sql

# 备份配置文件
cp /etc/sqlrustgo.toml /backup/sqlrustgo.toml.bak
```

### 2.2 停止服务

```bash
# 停止 SQLRustGo 服务
systemctl stop sqlrustgo

# 或直接停止进程
pkill -f sqlrustgo-server
```

### 2.3 更新代码

```bash
# 拉取最新代码
git fetch origin
git checkout develop/v2.1.0

# 编译新版本
cargo build --release
```

### 2.4 更新配置 (可选)

v2.1.0 新增配置项：

```toml
# etc/sqlrustgo.toml

# 新增: 可观测性配置
[observability]
prometheus_enabled = true
slow_query_threshold_ms = 1000

# 新增: SQL 防火墙配置
[firewall]
enabled = true
alert_on_suspicious = true
```

### 2.5 启动服务

```bash
# 启动服务
sqlrustgo-server --config /etc/sqlrustgo.toml

# 验证版本
sqlrustgo --version
# 输出: sqlrustgo 2.1.0
```

---

## 三、主要变更

### 3.1 MockStorage 已废弃

v2.1.0 已移除 MockStorage，所有测试使用 MemoryStorage。

**影响**: 无需用户操作

**测试验证**:
```bash
cargo test --test regression_test
```

### 3.2 新数据类型

v2.1.0 支持 UUID, ARRAY, ENUM 类型。

**使用示例**:
```sql
-- UUID
CREATE TABLE sessions (
    session_id UUID PRIMARY KEY,
    user_id INTEGER
);

-- ARRAY
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    values INTEGER ARRAY
);

-- ENUM
CREATE TYPE status AS ENUM ('pending', 'active', 'closed');
```

### 3.3 备份工具增强

新增保留策略功能：

```bash
# 保留最近7个备份
sqlrustgo-tools physical-backup prune --dir /backup --keep 7

# 保留最近30天
sqlrustgo-tools physical-backup prune --dir /backup --keep-days 30
```

### 3.4 SQL Firewall

新增 KILL 语句支持：

```sql
-- 查看进程
SHOW PROCESSLIST;

-- 终止进程
KILL 12345;
```

---

## 四、配置变更

### 4.1 新增配置项

```toml
[observability]
# Prometheus 指标端点
prometheus_enabled = true
metrics_port = 9090

# 慢查询阈值 (毫秒)
slow_query_threshold_ms = 1000

[firewall]
# SQL 防火墙启用
enabled = true
# 可疑查询告警
alert_on_suspicious = true
# 日志路径
alert_log_path = "/var/log/sqlrustgo/firewall_alerts.log"
```

### 4.2 配置迁移脚本

```bash
#!/bin/bash
# migrate_config.sh

OLD_CONFIG=/etc/sqlrustgo.toml
NEW_CONFIG=/etc/sqlrustgo.toml.new

# 添加新配置项
cat >> $NEW_CONFIG << 'EOF'

# === v2.1.0 新增配置 ===
[observability]
prometheus_enabled = true
metrics_port = 9090
slow_query_threshold_ms = 1000

[firewall]
enabled = true
alert_on_suspicious = true
alert_log_path = "/var/log/sqlrustgo/firewall_alerts.log"
EOF

echo "新配置已添加到 $NEW_CONFIG"
```

---

## 五、验证升级

### 5.1 版本验证

```bash
sqlrustgo --version
# 输出: sqlrustgo 2.1.0
```

### 5.2 功能验证

```bash
# 1. 健康检查
curl http://localhost:8080/health

# 2. Prometheus 指标
curl http://localhost:8080/metrics

# 3. 运行回归测试
cargo test --test regression_test

# 4. TPC-H 测试
cargo test --test tpch_test
```

### 5.3 性能验证

```bash
# 运行基准测试
cargo bench --bench tpch_benchmark

# 验证 QPS
# 目标: ≥1000 QPS (50并发)
```

---

## 六、回滚

### 6.1 回滚步骤

```bash
# 1. 停止服务
systemctl stop sqlrustgo

# 2. 恢复代码
git checkout v2.0.0

# 3. 重新编译
cargo build --release

# 4. 恢复数据
sqlrustgo-tools mysqldump --database mydb --in /backup/mydb.sql

# 5. 恢复配置
cp /backup/sqlrustgo.toml.bak /etc/sqlrustgo.toml

# 6. 启动服务
systemctl start sqlrustgo
```

### 6.2 数据兼容性

v2.1.0 与 v2.0 存储格式完全兼容，可以直接回滚。

---

## 七、常见问题

### Q: 升级后编译失败？
A: 确保 Rust 工具链是最新版本 `rustup update`

### Q: 测试失败怎么办？
A: 查看详细输出 `cargo test -- --nocapture`

### Q: 性能下降？
A: 运行 `cargo bench` 对比基准测试结果

---

*升级指南 v2.1.0*
