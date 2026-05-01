# v2.8.0 升级指南

> **版本**: v2.8.0
> **适用**: 从 v2.7.0 升级
> **日期**: 2026-05-02

---

## 概述

本指南帮助用户从 v2.7.0 升级到 v2.8.0。v2.8.0 是"分布式增强 + 安全加固"版本，包含分区表、主从复制、故障转移、读写分离、审计告警系统、SIMD 向量化加速等重大功能改进。

**当前状态**: v2.8.0 处于 Alpha 阶段，大多数 v2.7.0 SQL 语句无需修改即可运行。请仔细阅读本指南了解破坏性变更。

---

## 重大变更

### Breaking Changes

| 变更 | 影响 | 迁移建议 |
|------|------|----------|
| `ExecutionEngine::execute_raw` API 移除 | 需要改用 `execute` 方法 | 搜索并替换为 `execute` |
| `VectorSearch::search_knn` API 移除 | 需要改用 `UnifiedSearch::hybrid_search` | 见下方 API 变更章节 |
| `SearchAPI::legacy_search()` API 移除 | 需要改用 `SearchAPI::unified_search()` | 见下方 API 变更章节 |
| `[vector].hnsw_enable` 配置项移除 | 需要改用 `[vector].index_type` | 见配置变更章节 |
| `TableInfo` 新增 `check_constraints` 字段 | 自定义初始化代码需适配 | 见下方 Rust API 变更章节 |

### API 变更

#### TableInfo 新增 CheckConstraint 字段

```rust
// v2.8.0 新增
pub struct CheckConstraint {
    pub name: String,
    pub expression: String,
}

pub struct TableInfo {
    // ... 原有字段 ...
    pub check_constraints: Vec<CheckConstraint>,  // v2.8.0 新增
}
```

如果您自定义了 `TableInfo` 初始化代码，需要添加 `check_constraints` 字段：

```rust
// v2.7.0
TableInfo {
    name: "users".to_string(),
    columns: vec![],
    // ...
}

// v2.8.0
TableInfo {
    name: "users".to_string(),
    columns: vec![],
    check_constraints: vec![],  // 新增字段
    // ...
}
```

#### 已移除的废弃 API

| API | 替代方案 |
|-----|----------|
| `ExecutionEngine::execute_raw` | `ExecutionEngine::execute` |
| `VectorSearch::search_knn` | `UnifiedSearch::hybrid_search` |
| `SearchAPI::legacy_search()` | `SearchAPI::unified_search()` |

#### 新增分布式 API

v2.8.0 引入了分布式集群管理 API：

```rust
use sqlrustgo::distributed::ClusterManager;

// 创建集群管理器
let cluster = ClusterManager::new(config)?;

// 添加从节点
cluster.add_replica("replica1", "192.168.1.101:3307")?;

// 监控复制状态
let status = cluster.replication_status()?;

// 触发故障转移
cluster.initiate_failover()?;
```

#### 新增审计告警 API

```rust
use sqlrustgo::security::AuditManager;

// 创建审计管理器
let audit = AuditManager::new(config)?;

// 记录审计事件
audit.record_sql_execution("SELECT * FROM users", "admin")?;

// 查询审计日志
let logs = audit.query_logs(start_time, end_time)?;
```

---

## SQL 语法变更

### 新增功能

#### CHECK 约束

```sql
-- v2.8.0 新增支持 CHECK 约束
CREATE TABLE users (
    id INT PRIMARY KEY,
    age INT CHECK (age >= 0 AND age <= 150),
    name VARCHAR(100),
    salary DECIMAL(10,2) CHECK (salary > 0)
);

-- v2.7.0 中 CHECK 被解析但不会强制执行
-- v2.8.0 数据结构已支持，验证逻辑待完整实现
```

#### FULL OUTER JOIN（修复）

```sql
-- v2.7.0 可能不支持或结果不正确
-- v2.8.0 已修复，3/3 测试通过

SELECT a.id, a.name, b.order_id
FROM customers a
FULL OUTER JOIN orders b ON a.id = b.customer_id;
```

#### TRUNCATE TABLE

```sql
-- v2.8.0 新增支持
TRUNCATE TABLE users;

-- 清空表并重置自增计数器
TRUNCATE TABLE orders;
```

#### REPLACE INTO

```sql
-- v2.8.0 新增支持
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- 如果 id=1 存在则替换，否则插入
```

#### 窗口函数

```sql
-- v2.8.0 新增 ROW_NUMBER / RANK / DENSE_RANK
SELECT
    name,
    salary,
    ROW_NUMBER() OVER (ORDER BY salary DESC) AS row_num,
    RANK() OVER (ORDER BY salary DESC) AS rank,
    DENSE_RANK() OVER (ORDER BY salary DESC) AS dense_rank
FROM employees;
```

#### 分区表

```sql
-- v2.8.0 新增分区表支持
CREATE TABLE sales (
    id INT,
    amount DECIMAL(10,2),
    sale_date DATE
)
PARTITION BY RANGE (YEAR(sale_date)) (
    PARTITION p2024 VALUES LESS THAN (2025),
    PARTITION p2025 VALUES LESS THAN (2026),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);

-- LIST 分区
CREATE TABLE regions (
    id INT,
    region_name VARCHAR(50)
)
PARTITION BY LIST (id) (
    PARTITION p_north VALUES IN (1, 2, 3),
    PARTITION p_south VALUES IN (4, 5, 6)
);

-- HASH 分区
CREATE TABLE logs (
    id INT,
    message TEXT
)
PARTITION BY HASH (id) PARTITIONS 4;
```

### 行为变更

| 操作 | v2.7.0 | v2.8.0 |
|------|--------|--------|
| FULL OUTER JOIN | 不支持或结果不正确 | 基于 Hash 匹配算法，3/3 测试通过 |
| TRUNCATE TABLE | 不支持 | 支持（语法解析 + 执行） |
| REPLACE INTO | 不支持 | 支持（语法解析 + 执行） |
| CHECK 约束 | 被解析但不强制执行 | 数据结构支持（验证逻辑部分实现） |
| 窗口函数 | 仅 ROW_NUMBER/RANK | 完整 ROW_NUMBER/RANK/DENSE_RANK |
| 分区表 | 不支持 | Range/List/Hash/Key 四类 + 分区裁剪 |

### 性能改进

| 操作 | v2.7.0 | v2.8.0 | 提升 |
|------|--------|--------|------|
| 单元测试通过 | 210 | **258** | +22.9% |
| 分布式测试 | 0 | **658** | 新增 |
| 总测试规模 | 368 | **1090** | +196% |
| WAL 恢复测试 | 12 | **16** | +33% |
| SIMD 向量化 | 不支持 | ✅ 支持 | 基础加速 |

---

## 升级步骤

### 阶段 1: 准备

1. **备份所有数据库文件**

```bash
# 创建备份目录
mkdir -p /backup/v2.7.0

# 执行全量备份
sqlrustgo-tools backup --database <db> --output-dir /backup/v2.7.0 --backup-type full
```

2. **停止所有运行中的 SQLRustGo 实例**

```bash
# 检查运行中的实例
ps aux | grep sqlrustgo

# 优雅停止 (发送 SIGTERM)
kill -TERM <pid>
```

### 阶段 2: 升级

3. **切换到 develop/v2.8.0 分支**

```bash
git checkout develop/v2.8.0
```

4. **重新构建**

```bash
cargo build --release
```

或使用全特性构建：

```bash
cargo build --release --all-features
```

5. **替换旧 API 调用（如使用 Rust API）**

检查并更新以下代码：
- `execute_raw` → `execute`
- `search_knn` → `hybrid_search`
- `legacy_search` → `unified_search`
- `TableInfo` 初始化添加 `check_constraints: vec![]`

### 阶段 3: 配置

6. **更新配置项**

编辑配置文件 `sqlrustgo.toml`：

```toml
# v2.8.0 新增配置

# CHECK 约束验证（默认关闭）
check_constraint_validation = false

# 向量索引类型（替代已废弃的 hnsw_enable）
[vector]
index_type = "hnsw"  # 或 "flat", "ivf"

# 分布式配置（可选）
[distributed]
enabled = false
replication_factor = 2

[distributed.failover]
enabled = true
detection_timeout_ms = 5000
switch_timeout_ms = 30000

[distributed.load_balancing]
strategy = "round_robin"  # round_robin / least_connections
health_check_interval_sec = 30

# 审计配置（可选）
[audit]
enabled = true
storage_path = "/var/lib/sqlrustgo/audit"
retention_days = 365

# 备份配置（可选）
[backup]
schedule = "daily"
retention_days = 7
compression = true
remote_endpoint = ""
```

### 阶段 4: 验证

7. **验证数据完整性**

```bash
# 检查数据库状态
sqlrustgo-tools check --integrity

# 运行迁移测试
cargo test --test migration_tests
```

8. **启动新版本实例**

```bash
# 启动 sqlrustgo
sqlrustgo --config sqlrustgo.toml

# 或启动服务器模式
sqlrustgo-server --config sqlrustgo.toml
```

9. **验证核心功能**

```bash
# 运行基础回归测试
cargo test --test regression_test

# 运行崩溃恢复测试
cargo test --test crash_recovery_test

# 运行 WAL 恢复测试
cargo test --test wal_integration_test
```

10. **验证分布式能力（如启用）**

```bash
# 分区表测试
cargo test -p sqlrustgo-distributed -- partition

# 主从复制测试
cargo test -p sqlrustgo-distributed -- replication

# 故障转移测试
cargo test -p sqlrustgo-distributed -- failover

# 读写分离测试
cargo test -p sqlrustgo-distributed -- read_write
```

---

## 配置变更

### 构建要求

v2.8.0 需要以下依赖：

- Rust 1.70+（验证版本: 1.94.1）
- Cargo（随 Rust 安装）
- 内存 8GB+（部分测试需 16GB+）

### 特性标志

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 全特性构建
cargo build --all-features

# 分布式特性
cargo build --features distributed

# 安全模块
cargo build --features security

# AES-256 加密（feature-gated）
cargo build --features aes256
```

### 新增配置项

```toml
# v2.8.0 新增配置项
check_constraint_validation = false  # 是否启用 CHECK 约束验证

[vector]
index_type = "hnsw"  # 向量索引类型（替代 hnsw_enable）

[distributed]
enabled = false
replication_factor = 2

[distributed.failover]
enabled = true
detection_timeout_ms = 5000
switch_timeout_ms = 30000

[distributed.load_balancing]
strategy = "round_robin"
health_check_interval_sec = 30

[audit]
enabled = true
storage_path = "/var/lib/sqlrustgo/audit"
retention_days = 365

[audit.alert]
enabled = true
levels = ["info", "warning", "critical"]

[backup]
schedule = "daily"
retention_days = 7
compression = true
```

### 废弃/移除配置项

| 配置项 | 废弃版本 | 替代方案 | 移除版本 |
|--------|----------|----------|----------|
| `[vector].hnsw_enable` | v2.7.0 | `[vector].index_type` | **v2.8.0 (已移除)** |

---

## 数据迁移

### 存储格式

v2.8.0 与 v2.7.0 数据格式兼容。主要的存储变更集中在新增功能，不影响现有数据格式。

### 索引

升级后建议重建索引以获得最佳性能：

```sql
REINDEX DATABASE;
```

### 分区表

从 v2.7.0 升级后，如需使用分区表功能，需要重新创建表并迁移数据：

```sql
-- 创建分区表
CREATE TABLE sales_new (
    id INT,
    amount DECIMAL(10,2),
    sale_date DATE
)
PARTITION BY RANGE (YEAR(sale_date)) (...);

-- 迁移数据
INSERT INTO sales_new SELECT * FROM sales;

-- 重命名
DROP TABLE sales;
ALTER TABLE sales_new RENAME TO sales;
```

---

## 分布式集群配置（新增）

### 主从复制设置

```bash
# 主节点配置
sqlrustgo-server --config master.toml --server-id 1

# 从节点配置
sqlrustgo-server --config replica.toml --server-id 2

# 从节点连接到主节点（在 REPL 中）
CHANGE MASTER TO MASTER_HOST='192.168.1.100', MASTER_PORT=3306, MASTER_USER='repl', MASTER_PASSWORD='password';
START SLAVE;
```

### 故障转移设置

故障转移默认启用。配置示例：

```toml
[distributed.failover]
enabled = true
detection_timeout_ms = 5000   # 主节点宕机检测超时
switch_timeout_ms = 30000     # 自动切换最大时间
auto_rejoin = true            # 原主节点恢复后自动重新加入
```

---

## 已知问题

### 待解决 (v2.9.0)

| 问题 | 影响 | 优先级 |
|------|------|--------|
| Hash Join 并行化未集成 | 大规模 Join 性能受限 | P1 |
| 列级 GRANT/REVOKE 缺少解析器 | 权限管理不完整 | P0 |
| AES-256 未集成到存储管线 | 静态数据加密缺失 | P0 |
| SQL Corpus 通过率 40.8% | 函数调用语法不支持 | P0 |
| 33 个 `#[ignore]` 测试 | 边界条件未验证 | P0 |
| CHECK 约束验证逻辑 | 数据结构已添加，验证待完成 | P1 |
| 无 sysbench/TPC-H 基准测试 | 性能指标缺失 | P1 |
| 无 72h 实际长稳运行数据 | 加速模拟非真实时间 | P2 |

### 已知限制

- `SELECT COALESCE(a, b)` — 函数调用需大写 `FUNCTION`
- `SELECT CAST(a AS INT)` — CAST 语法不支持
- `SELECT CASE WHEN ... THEN ... END` — CASE 表达式不支持
- `SELECT (SELECT ...)` — 标量子查询不支持
- `SELECT RANK() OVER (...)` — 窗口函数调用不支持
- `GROUP BY ROLLUP/CUBE` — GROUP BY 扩展不支持
- 分布式事务协调（跨节点）仍在设计中
- SSI 隔离级别（可串行化快照隔离）仍在规划中

---

## 回滚

如果升级后遇到问题，可以回滚到 v2.7.0：

```bash
# 停止 v2.8.0 服务
kill -TERM <pid>

# 切换到 v2.7.0 标签
git checkout v2.7.0

# 重新构建
cargo build --release

# 从备份恢复数据
sqlrustgo-tools restore --database <db> --backup-id <id> --backup-dir /backup/v2.7.0

# 启动 v2.7.0 服务
sqlrustgo --config sqlrustgo.toml
```

**注意**: 如果升级后使用了分区表功能，回滚前需要将数据导出为 CSV/JSON/SQL 格式，回滚后再导入。

---

## 常见问题

### Q1: 升级后 `execute_raw` 编译错误？

从 v2.7.0 废弃的 `ExecutionEngine::execute_raw` API 已在 v2.8.0 移除。请改用 `ExecutionEngine::execute`：

```rust
// v2.7.0 (不再支持)
engine.execute_raw("SELECT * FROM users")?;

// v2.8.0
engine.execute("SELECT * FROM users")?;
```

### Q2: `[vector].hnsw_enable` 配置报错？

该配置项已在 v2.8.0 移除，请改用 `[vector].index_type`：

```toml
# v2.7.0 (不再支持)
[vector]
hnsw_enable = true

# v2.8.0
[vector]
index_type = "hnsw"
```

### Q3: 审计日志存储空间不足？

审计日志会自动清理超过 `retention_days` 的记录。如需手动清理：

```bash
sqlrustgo-tools clean-audit --before 2026-01-01
```

### Q4: CHECK 约束不生效？

`check_constraint_validation` 配置项默认关闭。如需启用：

```toml
check_constraint_validation = true
```

**注意**: v2.8.0 的 CHECK 约束数据结构已支持，但验证逻辑为部分实现。

### Q5: 分区表查询性能不佳？

确保分区裁剪已启用。可以通过 EXPLAIN 查看分区裁剪效果：

```sql
EXPLAIN SELECT * FROM sales WHERE sale_date = '2025-03-15';
```

### Q6: 升级后迁移测试失败？

确保已完成全量备份。然后查看具体错误信息：

```bash
cargo test --test migration_tests -- --nocapture
```

### Q7: 性能回退？

运行性能验证测试：

```bash
cargo test --test qps_benchmark_test
cargo test --test buffer_pool_benchmark_test
```

---

## 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [v2.8.0 文档入口](./README.md)
- [迁移指南](../../MIGRATION_GUIDE.md)
- [版本计划](./VERSION_PLAN.md)
- [安全加固指南](./SECURITY_HARDENING.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-02*
