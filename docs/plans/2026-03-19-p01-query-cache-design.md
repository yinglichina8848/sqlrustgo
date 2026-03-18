# P-01 查询缓存设计文档

> **版本**: v1.0
> **日期**: 2026-03-19
> **功能**: P-01 查询缓存
> **目标分支**: develop/v1.6.0

---

## 一、架构概览

```
SQL 查询
    ↓
[参数化处理] → 提取 SQL 骨架 + 参数值
    ↓
[缓存查询] → 检查 LRU 缓存
    ├── 命中 → 返回缓存结果
    └── 未命中 → 执行查询 → 写入缓存
```

## 二、核心数据结构

### 2.1 CacheKey

```rust
struct CacheKey {
    normalized_sql: String,  // 参数化 SQL（如 "SELECT * FROM t WHERE id = ?"）
    params_hash: u64,         // 参数值的哈希值
}
```

**为什么需要 params_hash？**

```sql
SELECT * FROM t WHERE id = 1;  -- key1
SELECT * FROM t WHERE id = 2;  -- key2

-- 如果只用 normalized_sql，两者会返回相同结果（错误！）
```

### 2.2 CacheEntry

```rust
struct CacheEntry {
    result: ResultSet,       // 查询结果
    tables: Vec<TableId>,    // 依赖的表（用于失效追踪）
    created_at: Instant,     // 创建时间（用于 TTL）
    size_bytes: usize,       // 结果大小（用于容量管理）
}
```

### 2.3 索引结构

```rust
// 主缓存
HashMap<CacheKey, CacheEntry>

// 表索引（用于快速失效）
HashMap<TableId, HashSet<CacheKey>>
```

## 三、缓存策略

### 3.1 什么查询可以缓存？

```rust
fn should_cache(plan: &Plan, result: &ResultSet) -> bool {
    // ✅ 可以缓存
    // - SELECT 语句
    // - 小结果集（< 1MB 或 < 1000 行）
    // - 确定性查询（无 NOW(), RAND() 等非确定函数）

    // ❌ 不缓存
    // - INSERT / UPDATE / DELETE
    // - 大结果集
    // - 非确定函数
}
```

### 3.2 缓存配置

```rust
struct QueryCacheConfig {
    max_entries: usize,      // 最大缓存条目（默认 1000）
    max_memory_bytes: usize,  // 最大内存使用（默认 100MB）
    ttl_seconds: u64,         // TTL（默认 30s）
    enabled: bool,           // 是否启用
}
```

## 四、失效机制

### 4.1 失效触发

当执行 INSERT/UPDATE/DELETE 时：

```rust
fn on_write_operation(table_id: TableId) {
    // 1. 找到该表相关的所有 CacheKey
    if let Some(keys) = table_index.get(&table_id) {
        for key in keys.clone() {
            // 2. 从主缓存删除
            cache.remove(&key);
        }
    }
    // 3. 清除表索引
    table_index.remove(&table_id);

    // 4. 记录 metrics
    metrics.record_invalidation(keys.len());
}
```

### 4.2 表索引维护

- **写入缓存时**：提取查询涉及的表，加入 `table_index`
- **失效时**：根据 `table_index` 快速定位并删除

## 五、LRU 淘汰策略

### 5.1 独立实现

**不复用 BufferPool**，因为：
- BufferPool 缓存的是数据页（Page）
- QueryCache 缓存的是查询结果（ResultSet）
- 两者数据结构、生命周期完全不同

### 5.2 LRU 实现

```rust
struct QueryCache {
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: Vec<CacheKey>,           // 访问顺序
    table_index: HashMap<TableId, HashSet<CacheKey>>,
    config: QueryCacheConfig,
}

impl QueryCache {
    fn get(&mut self, key: &CacheKey) -> Option<&ResultSet> { ... }
    fn put(&mut self, key: CacheKey, entry: CacheEntry) { ... }
    fn invalidate_table(&mut self, table_id: TableId) { ... }
    fn evict_if_needed(&mut self) { ... }  // LRU 淘汰
}
```

## 六、Metrics 统计

| Metric | 说明 |
|--------|------|
| `query_cache_hits` | 缓存命中次数 |
| `query_cache_misses` | 缓存未命中次数 |
| `query_cache_evictions` | LRU 淘汰次数 |
| `query_cache_invalidations` | 失效次数 |

## 七、集成位置

```
crates/executor/
├── src/
│   ├── lib.rs
│   ├── executor.rs
│   ├── query_cache.rs      # 新增
│   └── mod.rs
```

### 7.1 Executor 集成

```rust
// executor.rs
impl Executor {
    fn execute(&mut self, plan: &Plan) -> Result<ExecutorResult, SqlError> {
        let cache_key = self.build_cache_key(plan)?;

        // 1. 检查缓存
        if let Some(result) = self.cache.get(&cache_key) {
            metrics.record_hit();
            return Ok(result.clone());
        }

        // 2. 执行查询
        let result = self.execute_inner(plan)?;

        // 3. 判断是否缓存
        if should_cache(plan, &result) {
            self.cache.put(cache_key, CacheEntry::new(result.clone()));
        }

        Ok(result)
    }

    fn on_write(&mut self, table_id: TableId) {
        self.cache.invalidate_table(table_id);
    }
}
```

### 7.2 Storage 层触发失效

```rust
// storage/engine.rs
impl StorageEngine for Storage {
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        let result = self.do_insert(table, records)?;
        // 触发缓存失效
        self.executor.on_write(table_id);
        result
    }
    // ... update, delete 同理
}
```

## 八、实现顺序（3 天）

### Day 1：核心结构
- [ ] 定义 `CacheKey`, `CacheEntry`, `CacheConfig`
- [ ] 实现 `QueryCache::new()`, `get()`, `put()`
- [ ] 实现 LRU 淘汰逻辑
- [ ] 实现 `table_index` 维护

### Day 2：Executor 集成
- [ ] 在 `executor.rs` 中集成缓存
- [ ] 实现 `should_cache()` 策略
- [ ] 实现参数化 SQL 提取
- [ ] 实现 `params_hash` 计算

### Day 3：失效 + Metrics + 测试
- [ ] 实现表级失效机制
- [ ] 集成 Metrics 统计
- [ ] 单元测试
- [ ] 集成测试
- [ ] 创建 PR

## 九、风险与注意事项

| 风险 | 缓解措施 |
|------|----------|
| 大结果集撑爆内存 | `max_memory_bytes` 限制 + `should_cache()` 过滤 |
| 缓存过期数据 | TTL 30s + 表级失效 |
| 哈希冲突 | 使用 xxhash 或 FxHash，冲突概率极低 |

---

**评审状态**: ✅ 已批准
**下一步**: 创建实施计划
