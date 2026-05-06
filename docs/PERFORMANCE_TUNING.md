# SQLRustGo 性能调优指南

> **版本**: 1.0
> **适用版本**: v3.0.0
> **最后更新**: 2026-05-06

---

## 一、性能基准

### QPS 基准（MemoryStorage, 2026-05-06）

| 操作 | QPS | 硬件 |
|------|-----|------|
| Aggregation | 1,643,824 | macOS (Darwin 25.4.0) |
| ORDER BY | 81,988 | MemoryStorage |
| DELETE | 63,568 | 单线程 |
| JOIN | 57,388 | Hash Join |
| UPDATE | 43,224 | 原位操作 |
| INSERT | 33,377 | MemoryStorage |
| Simple SELECT | 24,516 | 全表扫描 |
| Concurrent SELECT | 11,995 | 8线程 |

---

## 二、Buffer Pool 调优

### 工作原理

Buffer Pool 是 SQLRustGo 的页缓存层（仅 FileStorage 模式生效）。

### 配置

```toml
[storage]
buffer_pool_size = 256  # MB, 默认 256MB
buffer_pool_instances = 4  # 分片数, 默认 4
```

### 调优建议

| 场景 | Buffer Pool | Instances |
|------|-------------|-----------|
| 开发/测试 | 64-256 MB | 1-4 |
| 生产 (8GB RAM) | 2-4 GB | 4-8 |
| 生产 (32GB RAM) | 8-16 GB | 8-16 |
| 生产 (64GB RAM) | 16-32 GB | 16 |

### 监控

```sql
-- 查看缓存命中率
SHOW STATUS LIKE 'Innodb_buffer_pool_read%';
```

**目标**: 缓存命中率 > 95%

---

## 三、WAL 调优

### 工作原理

WAL (Write-Ahead Logging) 确保事务持久性。所有写入先写日志再写数据页。

### 配置

```toml
[storage]
wal_buffer_size = 4  # MB

[storage.group_commit]
batch_size = 100      # 批量提交行数
timeout_ms = 10       # 批提交等待时间
```

### 调优建议

| 场景 | batch_size | timeout_ms | wal_buffer |
|------|-----------|------------|------------|
| 低写入 (< 1K QPS) | 10-50 | 1-5 | 2MB |
| 中等写入 (1-10K QPS) | 50-200 | 5-10 | 4MB |
| 高写入 (> 10K QPS) | 200-500 | 10-20 | 8-16MB |
| 批量导入 | 500-1000 | 20-50 | 64MB |

### 代价权衡

| batch_size | 写入吞吐 | 延迟 | fsync 次数 |
|-----------|---------|------|-----------|
| 1 | 基准 | 最低 | 最多 |
| 100 | 2-3x | ~10ms | 减少 100x |
| 1000 | 5-10x | ~50ms | 减少 1000x |

---

## 四、查询缓存

### 工作原理

查询缓存缓存 `SELECT` 查询结果。DML 操作后自动失效。

### 配置

```toml
[query_cache]
enabled = true
size = 256  # MB
```

### 适用场景

| 场景 | 收益 | 建议 |
|------|------|------|
| 重复查询多 | 高 | 启用，512MB+ |
| 写入密集 | 低 | 禁用或小缓存 |
| 混合工作负载 | 中 | 启用，256MB |

---

## 五、索引优化

### 创建索引

```sql
-- 单列索引
CREATE INDEX idx_users_name ON users(name);

-- 唯一索引
CREATE UNIQUE INDEX idx_users_email ON users(email);

-- 复合索引（用于多条件查询）
CREATE INDEX idx_users_age_name ON users(age, name);
```

### 索引使用注意事项

| 查询模式 | 推荐索引 | 说明 |
|---------|---------|------|
| `WHERE id = ?` | 主键索引 | 自动创建 |
| `WHERE name = ?` | B+Tree 索引 | `CREATE INDEX` |
| `WHERE age > 30` | B+Tree 索引 | 范围查询 |
| `WHERE name LIKE 'foo%'` | B+Tree 索引 | 前缀模糊 |
| `WHERE name LIKE '%foo%'` | 无索引 | 全表扫描 |

---

## 六、连接池

### 工作原理

连接池复用数据库连接，避免每次查询重新建立连接的开销。

### 配置

```toml
[server]
max_connections = 151
connection_pool_size = 100
```

### 调优建议

| 并发客户端 | 连接池大小 | 每个客户端连接数 |
|-----------|-----------|----------------|
| 1-10 | 10-20 | 2 |
| 10-50 | 50-100 | 2-5 |
| 50-200 | 100-151 | 5-10 |

---

## 七、查询优化

### 使用 EXPLAIN

```sql
-- 查看查询计划
EXPLAIN SELECT * FROM users WHERE id = 1;

-- 查看实际执行计划
EXPLAIN ANALYZE SELECT * FROM orders JOIN lineitem ON o_orderkey = l_orderkey;
```

### 常见优化模式

**1. 使用索引避免全表扫描**

```sql
-- 慢: 全表扫描
SELECT * FROM orders WHERE o_custkey = 12345;

-- 快: 索引查找
CREATE INDEX idx_orders_custkey ON orders(o_custkey);
SELECT * FROM orders WHERE o_custkey = 12345;
```

**2. 使用 JOIN 代替子查询**

```sql
-- 慢: 相关子查询
SELECT * FROM orders WHERE EXISTS (SELECT 1 FROM lineitem WHERE l_orderkey = o_orderkey);

-- 快: Hash Join
SELECT * FROM orders JOIN lineitem ON o_orderkey = l_orderkey;
```

**3. 选择性聚合**

```sql
-- 避免在大量数据上 COUNT(DISTINCT)
SELECT COUNT(*) FROM orders;  -- 快
SELECT COUNT(DISTINCT o_custkey) FROM orders;  -- 慢，全表扫描
```

---

## 八、硬件与操作系统

### 存储

| 类型 | 随机读 | 顺序写 | 推荐于 |
|------|--------|--------|--------|
| NVMe SSD | 10-50 us | 1-2 GB/s | ✅ 推荐 |
| SATA SSD | 50-200 us | 300-600 MB/s | ✅ 可接受 |
| HDD | 5-15 ms | 100-200 MB/s | ❌ 不推荐 |

### 内存

- Buffer Pool 应占用总可用内存的 50-70%
- 剩余内存用于 OS 页缓存和连接栈

### CPU

- 单线程查询：高频 CPU 比多核更重要
- 并发查询：多核有利（8-16 核推荐）

---

## 九、性能基准测试

### QPS 基准

```bash
# 运行全部 QPS 基准
cargo test --test qps_benchmark_test -- --ignored --nocapture

# 运行单项
cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture
```

### TPC-H 性能

```bash
# SF=0.1 (需要外部数据)
cargo run -p sqlrustgo-bench-cli -- tpch-bench --data ~/sqlrustgo-tpch/data --queries Q1 --iterations 3
```

### 回归检测

```bash
# 与基线对比
bash scripts/gate/check_regression.sh --skip-run
```

---

*本文档由 SQLRustGo Team 维护。更新日期: 2026-05-06*
