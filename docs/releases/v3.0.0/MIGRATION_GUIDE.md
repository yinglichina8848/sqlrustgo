# v2.9.0 → v3.0.0 迁移指南

> **版本**: v3.0.0
> **从**: v2.9.0
> **代号**: Performance Core
> **日期**: 2026-05-06

---

## 一、升级路径

### 兼容性说明

v3.0.0 是 **"从能用到好用"** 的分水岭版本，与 v2.9.0 相比：

- ✅ 向后兼容：大多数 v2.9.0 SQL 语句无需修改即可运行
- ⚠️ 性能目标大幅提升 (Point Select QPS ≥10,000)
- ⚠️ 新增 CBO、连接池、查询缓存等性能组件

---

## 二、主要变更

### 2.1 性能指标对比

| 指标 | v2.9.0 | v3.0.0 目标 | 提升 |
|------|---------|--------------|------|
| Point Select QPS | ~2,000 | ≥10,000 | 5x |
| UPDATE QPS | ~950 | ≥5,000 | 5x |
| DELETE QPS | ~206 | ≥2,000 | 10x |
| TPC-H SF=1 | 22/22 (慢) | 22/22 (p99<2s) | 显著 |

### 2.2 新增功能 (v3.0.0)

#### Phase 0 - Debt Sprint

| 功能 | 说明 | 兼容性 |
|------|------|--------|
| D-01 CBO 规则 | Predicate Pushdown, Projection Pruning, Constant Folding | ✅ 兼容 |
| D-02 CTE Planner | WITH 子句完整支持 | ✅ 兼容 |
| D-03 触发器 Storage | FileStorage 触发器支持 | ✅ 兼容 |

#### Phase 1 - Performance Pocket v1

| 功能 | 说明 | 兼容性 |
|------|------|--------|
| PP-01 CBO 完善 | Index Selection, Join Reordering | ✅ 兼容 |
| PP-02 连接池 | max_connections, connection_pool_size | ✅ 兼容 |
| PP-03 查询缓存 | DML 自动失效 | ✅ 兼容 |
| PP-04 Group Commit | WAL 批量 fsync | ✅ 兼容 |
| PP-05 批量 Insert | 1000 行 <100ms | ✅ 兼容 |

#### Phase 2 - SQL Completeness

| 功能 | 说明 | 兼容性 |
|------|------|--------|
| F-01 INSERT...SELECT | 基础 INSERT SELECT | ✅ 兼容 |
| F-02 窗口函数补全 | NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE | ✅ 兼容 |
| F-03 CTE 执行 | WITH + WITH RECURSIVE | ✅ 兼容 |
| F-04 SERIALIZABLE | Gap Locking | ⚠️ 需配置 |

#### Phase 3 - Infrastructure

| 功能 | 说明 | 兼容性 |
|------|------|--------|
| I-01 INFORMATION_SCHEMA | SHOW TABLES/COLUMNS 等效 | ✅ 兼容 |
| I-02 EXPLAIN ANALYZE | 完整执行计划 | ✅ 兼容 |
| I-03 SSL/TLS | MySQL 客户端 SSL 连接 | ⚠️ 需配置 |
| I-04 慢查询日志 | long_query_time 阈值 | ✅ 兼容 |

---

## 三、配置变更

### 3.1 新增配置项

```toml
# v3.0.0 新增性能配置
[performance]
# 连接池 (PP-02)
max_connections = 256
connection_pool_size = 16

# 查询缓存 (PP-03)
query_cache_size = 1000
query_cache_ttl_seconds = 300

# Group Commit (PP-04)
group_commit_batch_size = 32
group_commit_timeout_ms = 10

# CBO (PP-01)
enable_cbo = true
cbo_join_reordering = true
cbo_index_selection = true
```

### 3.2 行为变更

| 配置项 | v2.9.0 | v3.0.0 | 说明 |
|--------|---------|---------|------|
| `optimizer.default` | 无优化 | CBO 默认开启 | 可能影响执行计划 |
| `connection.pool_size` | 1 | 16 | 连接复用更高效 |

---

## 四、SQL 行为变更

### 4.1 CTE 语法

v2.9.0: 部分支持 WITH 子句
v3.0.0: 完整支持 WITH 和 WITH RECURSIVE

```sql
-- WITH RECURSIVE 现在可用
WITH RECURSIVE cte AS (
  SELECT 1 AS n
  UNION ALL
  SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;
```

### 4.2 窗口函数

v2.9.0: 仅 ROW_NUMBER, RANK, DENSE_RANK
v3.0.0: 新增 NTILE, LEAD, LAG, FIRST_VALUE, LAST_VALUE, NTH_VALUE

---

## 五、存储格式

### 5.1 向下兼容

v3.0.0 **完全向下兼容** v2.9.0 数据文件。

存储格式变更：
- 无存储格式变更
- WAL 格式保持兼容
- B+Tree 页格式保持兼容

---

## 六、API 变更

### 6.1 新增 API

| API | 说明 | 废弃替代 |
|-----|------|----------|
| `ExecutionEngine::with_cbo()` | 启用 CBO | - |
| `ConnectionPool::new(config)` | 连接池 | `Connection::new()` |
| `QueryCache::new(capacity)` | 查询缓存 | - |

### 6.2 废弃 API

| API | 废弃版本 | 替代 |
|-----|----------|------|
| `ExecutionEngine::new()` | 3.0.0 | `ExecutionEngine::with_config()` |
| `Optimizer::default()` | 3.0.0 | `Optimizer::new()` |

---

## 七、升级步骤

### 7.1 升级前

1. 备份数据
2. 记录当前 QPS 基线
3. 检查 v2.9.0 配置

### 7.2 升级

```bash
# 1. 更新依赖
cargo update

# 2. 构建
cargo build --all-features

# 3. 运行测试
cargo test --all-features

# 4. 启动并验证
cargo run --bin sqlrustgo
```

### 7.3 升级后

1. 验证 SQL 功能正常
2. 测量 QPS 提升
3. 检查慢查询日志

---

## 八、已知问题

| 问题 | 影响 | 解决方案 |
|------|------|----------|
| CBO 可能改变执行计划 | 查询结果不变，但计划不同 | 可通过 `enable_cbo = false` 禁用 |
| 连接池默认 16 | 连接数增加 | 如需保持旧行为，设置 `connection_pool_size = 1` |

---

*文档版本: v3.0.0*
*最后更新: 2026-05-06*