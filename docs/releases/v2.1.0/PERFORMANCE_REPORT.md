# SQLRustGo v2.1.0 性能报告

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、性能概述

v2.1.0 在保持 v2.0 性能水平的基础上，针对可观测性和工具链进行了优化。

---

## 二、基准测试结果

### 2.1 TPC-H 基准测试

| 测试 | 命令 | 结果 | 目标 | 状态 |
|------|------|------|------|------|
| tpch_test | `cargo test --test tpch_test` | 11 passed | 11 | ✅ |
| tpch_benchmark | `cargo test --test tpch_benchmark` | 12 passed | 12 | ✅ |
| tpch_full_test | `cargo test --test tpch_full_test` | 34 passed | 34 | ✅ |

### 2.2 QPS 基准

| 并发数 | 目标 QPS | 实际 QPS | 状态 |
|--------|----------|----------|------|
| 10 | ≥500 | ⬜ | ⬜ |
| 50 | ≥1000 | ⬜ | ⬜ |
| 100 | ≥800 | ⬜ | ⬜ |

### 2.3 延迟基准

| 百分位 | 目标 | 实际 | 状态 |
|--------|------|------|------|
| P50 | <50ms | ⬜ | ⬜ |
| P95 | <80ms | ⬜ | ⬜ |
| P99 | <100ms | ⬜ | ⬜ |

---

## 三、优化改进

### 3.1 LRU Cache 优化

| 指标 | 优化前 | 优化后 | 改进 |
|------|--------|--------|------|
| 时间复杂度 | O(n) | O(1) | ✅ |
| 内存追踪 | 无 | 精确 | ✅ |

**相关 Issue**: #1023

### 3.2 查询缓存

```rust
pub struct QueryCache {
    cache: HashMap<QueryKey, QueryResult>,
    stats: QueryCacheStats,
}

impl QueryCache {
    pub fn get(&self, key: &QueryKey) -> Option<&QueryResult>;
    pub fn insert(&mut self, key: QueryKey, result: QueryResult);
    pub fn invalidate_table(&mut self, table: &str);
}
```

### 3.3 并行执行

```rust
pub struct ParallelExecutor {
    scheduler: TaskScheduler,
    workers: usize,
}

impl ParallelExecutor {
    pub fn execute_parallel(&self, plan: PhysicalPlan) -> Result<ExecutionResult, SqlError>;
}
```

---

## 四、内存使用

### 4.1 内存基线

| 场景 | 内存使用 | 说明 |
|------|----------|------|
| 空闲 | <100MB | 仅服务运行 |
| 10 连接 | <200MB | 常规负载 |
| 100 连接 | <500MB | 高并发 |
| 峰值 | <2GB | 最大负载 |

### 4.2 Buffer Pool

| 配置 | 默认值 | 说明 |
|------|--------|------|
| max_pages | 10000 | 最大页数 |
| page_size | 8KB | 页大小 |
| replacer | Clock | 页面淘汰 |

---

## 五、存储性能

### 5.1 MemoryStorage

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| INSERT | O(1) | 哈希表插入 |
| SELECT | O(1) | 主键查找 |
| UPDATE | O(1) | 哈希表更新 |
| DELETE | O(1) | 哈希表删除 |
| SCAN | O(n) | 全表扫描 |

### 5.2 备份性能

| 操作 | 速度 | 说明 |
|------|------|------|
| 全量备份 | ~50MB/s | 取决于磁盘 |
| 增量备份 | ~100MB/s | 仅 WAL |
| 压缩备份 | ~20MB/s | GZIP 压缩 |

---

## 六、测试环境

### 6.1 硬件配置

| 组件 | 规格 |
|------|------|
| CPU | Intel i7-10700 / 8 核 |
| 内存 | 32GB DDR4 |
| 磁盘 | NVMe SSD 512GB |
| 网络 | 1 Gbps |

### 6.2 软件配置

| 软件 | 版本 |
|------|------|
| Rust | 1.75+ |
| OS | macOS 14 / Ubuntu 22.04 |
| Cargo | 1.75+ |

---

## 七、性能问题排查

### 7.1 慢查询分析

```bash
# 查看慢查询日志
tail -f /var/log/sqlrustgo/slow_query.log

# 分析慢查询
# 1. 检查是否有全表扫描
# 2. 检查索引使用情况
# 3. 检查查询复杂度
```

### 7.2 性能剖析

```bash
# 使用 perf 分析
perf record -g cargo run --release --bin sqlrustgo
perf report

# 使用火焰图
cargo flamegraph --bin sqlrustgo
```

---

## 八、性能目标

### 8.1 v2.1.0 目标

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| QPS (50并发) | ≥1000 | ⬜ |
| P99 延迟 | <100ms | ⬜ |
| 内存空闲 | <100MB | ⬜ |
| 备份速度 | ≥50MB/s | ⬜ |

### 8.2 v2.2 目标

| 指标 | 目标 | 改进方向 |
|------|------|----------|
| QPS (100并发) | ≥2000 | 并行执行优化 |
| P99 延迟 | <50ms | 查询缓存 |
| 启动时间 | <1s | 延迟加载 |

---

## 九、建议

### 9.1 配置优化

```toml
[server]
workers = 8  # 根据 CPU 核心数调整

[storage]
type = "memory"

[query_cache]
enabled = true
max_entries = 10000
ttl_seconds = 300
```

### 9.2 SQL 优化建议

1. **使用索引**
   - 主键查询自动使用索引
   - 定期运行 ANALYZE 更新统计信息

2. **避免全表扫描**
   - 使用 LIMIT 限制结果集
   - 使用 WHERE 条件过滤

3. **批量操作**
   - 使用批量 INSERT
   - 避免频繁小事务

---

*性能报告 v2.1.0*
