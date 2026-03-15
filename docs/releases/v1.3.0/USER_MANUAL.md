# SQLRustGo v1.3.0 用户手册

> 版本：v1.3.0
> 发布日期：2026-03-15
> 适用用户：数据库使用者、应用开发者、数据库学习者

---

## 一、快速开始

### 1.1 安装

#### 从源码构建

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建 (Debug 模式)
cargo build

# 构建 (Release 模式，推荐)
cargo build --release

# 运行测试
cargo test --workspace
```

#### 使用 Docker

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v1.3.0

# 运行 REPL
docker run -it minzuuniversity/sqlrustgo:v1.3.0

# 运行服务器模式
docker run -p 5432:5432 minzuuniversity/sqlrustgo:v1.3.0 --server
```

#### 使用预编译二进制

```bash
# macOS (Apple Silicon)
curl -L -o sqlrustgo-v1.3.0-darwin-arm64.tar.gz \
  https://github.com/minzuuniversity/sqlrustgo/releases/download/v1.3.0/sqlrustgo-v1.3.0-darwin-arm64.tar.gz
tar -xzf sqlrustgo-v1.3.0-darwin-arm64.tar.gz
./sqlrustgo-v1.3.0-darwin-arm64/sqlrustgo

# Linux (x86_64)
curl -L -o sqlrustgo-v1.3.0-linux-x86_64.tar.gz \
  https://github.com/minzuuniversity/sqlrustgo/releases/download/v1.3.0/sqlrustgo-v1.3.0-linux-x86_64.tar.gz
tar -xzf sqlrustgo-v1.3.0-linux-x86_64.tar.gz
./sqlrustgo-v1.3.0-linux-x86_64/sqlrustgo

# Windows
# 从 GitHub Releases 页面下载 .zip 文件
```

---

### 1.2 启动模式

SQLRustGo 支持两种运行模式：

| 模式 | 命令 | 说明 |
|------|------|------|
| **REPL 模式** | `cargo run` 或 `./sqlrustgo` | 交互式命令行 |
| **服务器模式** | `cargo run -- --server` 或 `./sqlrustgo --server` | Client-Server 架构，支持 TCP 连接 |

---

## 二、REPL 模式

### 2.1 启动 REPL

```bash
cargo run
# 或
./sqlrustgo
```

```
SQLRustGo v1.3.0
Type 'help' for commands, 'exit' to quit.

sql>
```

### 2.2 基本命令

| 命令 | 说明 |
|------|------|
| `help` | 显示帮助信息 |
| `exit` 或 `quit` | 退出 REPL |
| `version` | 显示版本信息 |

### 2.3 SQL 支持

v1.3.0 支持以下 SQL 操作：

#### 数据定义 (DDL)

```sql
-- 创建表
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);

-- 删除表
DROP TABLE users;
```

#### 数据操作 (DML)

```sql
-- 插入数据
INSERT INTO users (id, name, age) VALUES (1, 'Alice', 30);
INSERT INTO users (id, name, age) VALUES (2, 'Bob', 25);

-- 查询数据
SELECT * FROM users;
SELECT id, name FROM users WHERE age > 20;

-- 更新数据
UPDATE users SET age = 31 WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE id = 2;
```

#### 聚合查询

```sql
-- 计数
SELECT COUNT(*) FROM users;

-- 求和
SELECT SUM(age) FROM users;

-- 平均值
SELECT AVG(age) FROM users;

-- 分组
SELECT age, COUNT(*) FROM users GROUP BY age;
```

#### 连接查询

```sql
-- 内连接
SELECT * FROM users JOIN orders ON users.id = orders.user_id;

-- 左连接
SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id;
```

---

## 三、服务器模式

### 3.1 启动服务器

```bash
# 默认端口 5432
cargo run -- --server

# 自定义端口
cargo run -- --server --port 5433
```

服务器启动输出：

```
SQLRustGo v1.3.0 Server
Listening on 0.0.0.0:5432
```

### 3.2 连接服务器

```bash
# 使用内置客户端
cargo run --bin sqlrustgo-client -- --host localhost --port 5432

# 或使用 MySQL 客户端
mysql -h localhost -P 5432 -u root -p
```

### 3.3 服务器命令

| 命令 | 说明 |
|------|------|
| `--server` | 启动服务器模式 |
| `--port <port>` | 指定端口 (默认 5432) |
| `--host <host>` | 绑定地址 (默认 0.0.0.0) |

---

## 四、可观测性功能 (v1.3.0 新增)

### 4.1 健康检查端点

v1.3.0 提供 HTTP 健康检查端点：

#### 存活探针 (/health/live)

```bash
curl http://localhost:5432/health/live
```

响应：
```json
{
  "status": "alive",
  "version": "1.3.0"
}
```

#### 就绪探针 (/health/ready)

```bash
curl http://localhost:5432/health/ready
```

响应：
```json
{
  "status": "ready",
  "checks": {
    "storage": {"status": "healthy", "latency_ms": 5},
    "memory": {"status": "healthy", "usage_percent": 45.2},
    "connections": {"status": "healthy", "active": 10, "max": 100}
  }
}
```

#### 综合健康 (/health)

```bash
curl http://localhost:5432/health
```

响应：
```json
{
  "status": "healthy",
  "timestamp": "2026-03-15T10:00:00Z",
  "uptime_seconds": 3600,
  "components": {
    "storage": {"status": "healthy"},
    "executor": {"status": "healthy"},
    "network": {"status": "healthy"}
  },
  "metrics": {
    "queries_total": 1000,
    "queries_failed": 5,
    "avg_query_ms": 25.5
  }
}
```

### 4.2 指标端点 (/metrics)

v1.3.0 支持 Prometheus 格式指标：

```bash
curl http://localhost:5432/metrics
```

响应示例：
```prometheus
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total 1000

# TYPE sqlrustgo_connections_active gauge
sqlrustgo_connections_active 10

# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.001"} 500
```

### 4.3 可用指标

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `sqlrustgo_queries_total` | Counter | 总查询数 |
| `sqlrustgo_queries_failed_total` | Counter | 失败查询数 |
| `sqlrustgo_query_duration_seconds` | Histogram | 查询耗时 |
| `sqlrustgo_connections_active` | Gauge | 活跃连接数 |
| `sqlrustgo_connections_total` | Counter | 总连接数 |
| `sqlrustgo_bytes_sent_total` | Counter | 发送字节数 |
| `sqlrustgo_bytes_received_total` | Counter | 接收字节数 |
| `sqlrustgo_buffer_pool_hits_total` | Counter | 缓存命中数 |
| `sqlrustgo_buffer_pool_misses_total` | Counter | 缓存未命中数 |

---

## 五、监控集成 (v1.3.0 新增)

### 5.1 Prometheus 集成

1. 配置 Prometheus 抓取指标：

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'sqlrustgo'
    static_configs:
      - targets: ['localhost:5432']
```

2. 重启 Prometheus

### 5.2 Grafana 集成

1. 导入 Grafana Dashboard：`docs/monitoring/grafana-dashboard.json`
2. 配置 Prometheus 数据源
3. 查看仪表盘

仪表盘包含：
- QPS (每秒查询数)
- 平均查询延迟
- 缓存命中率
- 存储 I/O
- 网络流量

### 5.3 告警规则

v1.3.0 提供 Prometheus 告警规则：`docs/monitoring/prometheus-alerts.yml`

告警包括：
- HighQueryLatency: 查询延迟 > 1s
- HighErrorRate: 错误率 > 5%
- LowBufferPoolHit: 缓存命中率 < 80%

---

## 六、性能基准

### 6.1 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench bench_v130

# 查看结果
ls target/criterion/
```

### 6.2 基准测试覆盖

v1.3.0 提供以下基准测试：

| 类别 | 测试项 |
|------|--------|
| TableScan | 100/1K/10K 行扫描 |
| Filter | 等值/范围/AND 条件 |
| HashJoin | Inner/Left/Cross Join |

### 6.3 性能目标

| 操作 | 目标 | 实际 |
|------|------|------|
| INSERT 100k rows | < 2s | ~1.5s |
| SELECT * (100k rows) | < 200ms | ~150ms |
| HashJoin (100k × 100k) | < 2s | ~1.8s |

---

## 七、配置参考

### 7.1 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SQLRUSTGO_PORT` | 5432 | 服务器端口 |
| `SQLRUSTGO_HOST` | 0.0.0.0 | 绑定地址 |
| `SQLRUSTGO_BUFFER_POOL_SIZE` | 1000 | 缓冲池大小 |

### 7.2 命令行选项

```bash
sqlrustgo --help

# 输出：
# SQLRustGo v1.3.0
#
# Usage: sqlrustgo [OPTIONS]
#
# Options:
#   --server          Start in server mode
#   --port <PORT>    Server port (default: 5432)
#   --host <HOST>    Server host (default: 0.0.0.0)
#   --help           Show this help message
#   --version        Show version information
```

---

## 八、故障排查

### 8.1 常见问题

| 问题 | 解决方案 |
|------|----------|
| 编译失败 | 确保 Rust 工具链最新：`rustup update` |
| 测试失败 | 检查依赖：`cargo test --workspace` |
| 端口占用 | 更换端口：`--port 5433` |
| 内存不足 | 减少缓冲池：`SQLRUSTGO_BUFFER_POOL_SIZE=500` |

### 8.2 获取帮助

- 文档：`docs/`
- 问题反馈：https://github.com/minzuuniversity/sqlrustgo/issues
- 讨论：https://github.com/minzuuniversity/sqlrustgo/discussions

---

## 九、版本信息

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.3.0 | 2026-03-15 | Executor 稳定化 + 可观测性 |
| v1.2.0 | 2026-03-13 | 架构接口化 |
| v1.1.0 | 2026-03-05 | Client-Server 基础 |
| v1.0.0 | 2026-02 | 初始版本 |

---

*文档版本: 1.0*
*最后更新: 2026-03-15*
