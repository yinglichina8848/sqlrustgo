# Sysbench 测试环境设置与使用指南

> 本文档说明如何在 SQLRustGo 上设置和运行 sysbench OLTP 测试，用于验证数据库性能和兼容性。

## 1. 环境要求

### 1.1 安装 Sysbench

**macOS (Homebrew):**
```bash
brew install sysbench
```

**Ubuntu/Debian:**
```bash
sudo apt-get install sysbench
```

**验证安装:**
```bash
sysbench --version
# sysbench 1.0.20
```

### 1.2 安装 MySQL 客户端（可选）

用于手动连接测试:
```bash
brew install mysql-client
# 或
sudo apt-get install mysql-client
```

## 2. 启动 SQLRustGo 服务器

### 2.1 构建项目

```bash
cargo build --release -p sqlrustgo-mysql-server
```

### 2.2 启动服务器

```bash
# 前台运行
cargo run -p sqlrustgo-mysql-server -- --host 127.0.0.1 --port 3306

# 后台运行
cargo run -p sqlrustgo-mysql-server -- --host 127.0.0.1 --port 3306 > /tmp/sqlrustgo.log 2>&1 &

# 验证服务器运行
lsof -i :3306
```

## 3. 运行 Sysbench 测试

### 3.1 查找 Lua 脚本

**macOS (Homebrew):**
```bash
SYSBENCH_SCRIPTS=/opt/homebrew/share/sysbench
```

**Linux:**
```bash
SYSBENCH_SCRIPTS=/usr/local/share/sysbench
```

### 3.2 准备测试数据

```bash
SYSBENCH_SCRIPTS=/opt/homebrew/share/sysbench
cd /tmp

# 创建测试表（1张表，10000行）
sysbench $SYSBENCH_SCRIPTS/oltp_common.lua prepare \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --tables=1 \
  --table_size=10000
```

### 3.3 运行 OLTP 测试

**只读测试:**
```bash
SYSBENCH_SCRIPTS=/opt/homebrew/share/sysbench
cd /tmp

sysbench $SYSBENCH_SCRIPTS/oltp_read_only.lua run \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --threads=4 \
  --time=10
```

**只写测试:**
```bash
sysbench $SYSBENCH_SCRIPTS/oltp_write_only.lua run \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --threads=4 \
  --time=10
```

**混合读写测试:**
```bash
sysbench $SYSBENCH_SCRIPTS/oltp_read_write.lua run \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --threads=4 \
  --time=10
```

**点查询测试:**
```bash
sysbench $SYSBENCH_SCRIPTS/oltp_point_select.lua run \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --threads=4 \
  --time=10
```

### 3.4 清理测试数据

```bash
sysbench $SYSBENCH_SCRIPTS/oltp_common.lua cleanup \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306
```

## 4. CI/CD 集成

### 4.1 快速验证脚本

创建 `scripts/sysbench_smoke_test.sh`:

```bash
#!/bin/bash
set -e

SYSBENCH_SCRIPTS=${SYSBENCH_SCRIPTS:-/opt/homebrew/share/sysbench}
HOST=${MYSQL_HOST:-127.0.0.1}
PORT=${MYSQL_PORT:-3306}
THREADS=${THREADS:-4}
TIME=${TIME:-10}

echo "=== Sysbench Smoke Test ==="
echo "Host: $HOST:$PORT"
echo "Threads: $THREADS"
echo "Duration: ${TIME}s"

# Prepare
sysbench $SYSBENCH_SCRIPTS/oltp_common.lua prepare \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --tables=1 \
  --table_size=1000

# Run tests
echo "=== Running oltp_read_only ==="
sysbench $SYSBENCH_SCRIPTS/oltp_read_only.lua run \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --threads=$THREADS \
  --time=$TIME

echo "=== Running oltp_write_only ==="
sysbench $SYSBENCH_SCRIPTS/oltp_write_only.lua run \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --threads=$THREADS \
  --time=$TIME

echo "=== Running oltp_read_write ==="
sysbench $SYSBENCH_SCRIPTS/oltp_read_write.lua run \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --threads=$THREADS \
  --time=$TIME

# Cleanup
sysbench $SYSBENCH_SCRIPTS/oltp_common.lua cleanup \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT

echo "=== All tests passed ==="
```

### 4.2 GitHub Actions 示例

```yaml
name: Sysbench OLTP Test

on:
  push:
    branches: [develop/*, main]
  pull_request:
    branches: [develop/*, main]

jobs:
  sysbench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install sysbench
        run: |
          sudo apt-get update
          sudo apt-get install -y sysbench

      - name: Build SQLRustGo
        run: cargo build --release -p sqlrustgo-mysql-server

      - name: Start server
        run: |
          cargo run -p sqlrustgo-mysql-server -- --host 127.0.0.1 --port 3306 &
          sleep 5

      - name: Run sysbench tests
        run: |
          SYSBENCH_SCRIPTS=/usr/local/share/sysbench
          chmod +x scripts/sysbench_smoke_test.sh
          ./scripts/sysbench_smoke_test.sh
```

## 5. 测试结果解读

### 5.1 关键指标

| 指标 | 说明 | 良好值 |
|------|------|--------|
| QPS | 每秒查询数 | >10,000 |
| TPS | 每秒事务数 | >1,000 |
| 延迟 (avg) | 平均响应时间 | <10ms |
| 延迟 (p95) | 95th 百分位延迟 | <50ms |

### 5.2 期望结果（SQLRustGo v3.0.0）

| 测试场景 | QPS | TPS | 延迟 (avg) |
|----------|-----|-----|------------|
| oltp_read_only | ~17,000 | ~1,000 | ~4ms |
| oltp_write_only | ~37,000 | ~6,000 | ~0.7ms |
| oltp_read_write | ~19,000 | ~1,000 | ~4ms |

## 6. 故障排除

### 6.1 连接被拒绝

```bash
# 检查服务器是否运行
lsof -i :3306

# 检查端口是否可达
telnet 127.0.0.1 3306
```

### 6.2 认证失败

SQLRustGo 使用默认空密码。如需认证配置，参考 `docs/AUTHENTICATION.md`。

### 6.3 性能问题

```bash
# 检查服务器日志
cat /tmp/sqlrustgo.log

# 使用 debug 日志级别
RUST_LOG=debug cargo run -p sqlrustgo-mysql-server -- --log-level debug
```

## 7. 相关文档

- [MySQL Wire Protocol 修复指南](SYSBENCH_WIRE_PROTOCOL_FIX_GUIDE.md)
- [性能调优指南](../PERFORMANCE_TUNING.md)
- [慢查询日志](../crates/query-stats/src/slow_query_log.rs)
