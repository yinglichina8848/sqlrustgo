# P-01 查询缓存实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 Executor 层实现查询结果缓存，支持 LRU 淘汰、表级失效、TTL

**Architecture:** 在 `LocalExecutor` 外部封装 `QueryCache`，通过 CacheKey（参数化 SQL + 参数哈希）定位缓存

**Tech Stack:** Rust, LRU, xxhash

---

## 文件结构

```
crates/executor/src/
├── query_cache.rs      # 新增: QueryCache 核心实现
├── query_cache_config.rs  # 新增: 配置
├── query_cache_metrics.rs # 新增: Metrics 集成
├── lib.rs             # 修改: 导出新模块
└── local_executor.rs  # 修改: 集成缓存
```

---

## Day 1: 核心结构

### Task 1.1: 创建 QueryCacheConfig

**Files:**
- Create: `crates/executor/src/query_cache_config.rs`

**Step 1: 创建配置结构**

```rust
#[derive(Debug, Clone)]
pub struct QueryCacheConfig {
    pub max_entries: usize,
    pub max_memory_bytes: usize,
    pub ttl_seconds: u64,
    pub enabled: bool,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ttl_seconds: 30,
            enabled: true,
        }
    }
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/query_cache_config.rs
git commit -m "feat: add QueryCacheConfig"
```

---

### Task 1.2: 创建 CacheKey 和 CacheEntry

**Files:**
- Modify: `crates/executor/src/query_cache_config.rs`

**Step 1: 添加数据结构**

```rust
use std::time::{Duration, Instant};
use sqlrustgo_types::Value;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub normalized_sql: String,
    pub params_hash: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub result: crate::ExecutorResult,
    pub tables: Vec<String>,
    pub created_at: Instant,
    pub size_bytes: usize,
}

impl CacheEntry {
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    pub fn estimate_size(&self) -> usize {
        let mut size = 0;
        for row in &self.result.rows {
            for val in row {
                size += val.estimate_memory_size();
            }
        }
        size
    }
}
```

**Step 2: 添加 Value 内存估算**

在 `sqlrustgo_types` 中添加 `estimate_memory_size` 方法（如果不存在）

**Step 3: 提交**

```bash
git add crates/executor/src/query_cache_config.rs
git commit -m "feat: add CacheKey and CacheEntry structures"
```

---

### Task 1.3: 实现 QueryCache LRU

**Files:**
- Create: `crates/executor/src/query_cache.rs`

**Step 1: 创建 QueryCache 结构**

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use super::{CacheKey, CacheEntry, QueryCacheConfig};

pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    table_index: HashMap<String, HashSet<CacheKey>>,
    current_memory_bytes: usize,
}

impl QueryCache {
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            lru_order: VecDeque::new(),
            table_index: HashMap::new(),
            current_memory_bytes: 0,
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<crate::ExecutorResult> {
        if !self.config.enabled {
            return None;
        }

        let entry = self.cache.get_mut(key)?;
        
        // 检查 TTL
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        if entry.is_expired(ttl) {
            self.remove(key);
            return None;
        }

        // 更新 LRU
        self.touch(key);
        
        Some(entry.result.clone())
    }

    pub fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>) {
        if !self.config.enabled {
            return;
        }

        // 计算大小
        let size = entry.estimate_size();

        // 如果已存在，先删除
        if self.cache.contains_key(&key) {
            self.remove(&key);
        }

        // 检查容量
        while self.should_evict(size) {
            if let Some(oldest) = self.lru_order.pop_front() {
                self.remove(&oldest);
            } else {
                break;
            }
        }

        // 插入
        self.cache.insert(key.clone(), entry);
        self.lru_order.push_back(key.clone());
        self.current_memory_bytes += size;

        // 更新表索引
        for table in &tables {
            self.table_index
                .entry(table.clone())
                .or_insert_with(HashSet::new)
                .insert(key.clone());
        }
    }

    pub fn invalidate_table(&mut self, table: &str) {
        if let Some(keys) = self.table_index.remove(table) {
            for key in keys {
                if let Some(entry) = self.cache.remove(&key) {
                    self.current_memory_bytes -= entry.size_bytes;
                }
                // 从 lru_order 移除
                self.lru_order.retain(|k| k != &key);
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_order.clear();
        self.table_index.clear();
        self.current_memory_bytes = 0;
    }

    pub fn stats(&self) -> QueryCacheStats {
        QueryCacheStats {
            entries: self.cache.len(),
            memory_bytes: self.current_memory_bytes,
            table_count: self.table_index.len(),
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        self.lru_order.retain(|k| k != key);
        self.lru_order.push_back(key.clone());
    }

    fn remove(&mut self, key: &CacheKey) {
        if let Some(entry) = self.cache.remove(key) {
            self.current_memory_bytes -= entry.size_bytes;
        }
        self.lru_order.retain(|k| k != key);
        
        for table_keys in self.table_index.values_mut() {
            table_keys.remove(key);
        }
    }

    fn should_evict(&self, new_size: usize) -> bool {
        self.cache.len() >= self.config.max_entries
            || self.current_memory_bytes + new_size > self.config.max_memory_bytes
    }
}

#[derive(Debug, Clone)]
pub struct QueryCacheStats {
    pub entries: usize,
    pub memory_bytes: usize,
    pub table_count: usize,
}
```

**Step 2: 添加 Send + Sync**

```rust
unsafe impl Send for QueryCache {}
unsafe impl Sync for QueryCache {}
```

**Step 3: 提交**

```bash
git add crates/executor/src/query_cache.rs
git commit -m "feat: implement QueryCache with LRU"
```

---

### Task 1.4: 导出模块

**Files:**
- Modify: `crates/executor/src/lib.rs:3-18`

**Step 1: 添加导出**

```rust
pub mod query_cache;
pub mod query_cache_config;
pub mod query_cache_metrics;

pub use query_cache::{QueryCache, QueryCacheStats};
pub use query_cache_config::{CacheKey, CacheEntry, QueryCacheConfig};
pub use query_cache_metrics::QueryCacheMetrics;
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-executor --lib
```

**Step 3: 提交**

```bash
git add crates/executor/src/lib.rs
git commit -m "feat: export query cache modules"
```

---

## Day 2: Executor 集成

### Task 2.1: 实现参数化 SQL 提取

**Files:**
- Create: `crates/executor/src/sql_normalizer.rs`

**Step 1: 创建 SQL 归一化函数**

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use sqlrustgo_types::Value;

pub struct SqlNormalizer;

impl SqlNormalizer {
    pub fn normalize(sql: &str) -> String {
        let mut result = String::with_capacity(sql.len());
        let mut in_string = false;
        let mut chars = sql.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\'' {
                in_string = !in_string;
                result.push('?');
            } else if in_string {
                continue;
            } else if c.is_ascii_digit() {
                result.push('?');
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }

        result.trim().to_string()
    }

    pub fn hash_params(params: &[Value]) -> u64 {
        let mut hasher = DefaultHasher::new();
        params.hash(&mut hasher);
        hasher.finish()
    }

    pub fn from_literal(sql: &str) -> (String, Vec<Value>) {
        let mut params = Vec::new();
        let mut normalized = String::with_capacity(sql.len());
        let mut in_string = false;
        let mut current_param = String::new();

        for c in sql.chars() {
            if c == '\'' {
                if in_string {
                    params.push(Value::Text(current_param.clone()));
                    current_param.clear();
                }
                in_string = !in_string;
            } else if in_string {
                current_param.push(c);
            } else if c.is_ascii_digit() || c == '.' {
                current_param.push(c);
            } else {
                if !current_param.is_empty() {
                    if let Ok(n) = current_param.parse::<i64>() {
                        params.push(Value::Integer(n));
                    } else if let Ok(f) = current_param.parse::<f64>() {
                        params.push(Value::Float(f));
                    }
                    current_param.clear();
                }
                normalized.push(c.to_ascii_lowercase());
            }
        }

        if !current_param.is_empty() {
            if let Ok(n) = current_param.parse::<i64>() {
                params.push(Value::Integer(n));
            }
        }

        (normalized.trim().to_string(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_simple() {
        assert_eq!(
            SqlNormalizer::normalize("SELECT * FROM t WHERE id = 1"),
            "select * from t where id = ?"
        );
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            SqlNormalizer::normalize("SELECT * FROM t WHERE name = 'Alice'"),
            "select * from t where name = ?"
        );
    }

    #[test]
    fn test_from_literal() {
        let (sql, params) = SqlNormalizer::from_literal("SELECT * FROM t WHERE id = 42");
        assert_eq!(sql, "select * from t where id = ?");
        assert_eq!(params, vec![Value::Integer(42)]);
    }
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/sql_normalizer.rs
git commit -m "feat: add SQL normalizer for cache key generation"
```

---

### Task 2.2: 实现 should_cache 策略

**Files:**
- Modify: `crates/executor/src/query_cache.rs`

**Step 1: 添加 should_cache 函数**

```rust
const MAX_RESULT_SIZE_BYTES: usize = 1024 * 1024; // 1MB
const MAX_RESULT_ROWS: usize = 1000;

pub fn should_cache(result: &ExecutorResult) -> bool {
    // 不缓存空结果
    if result.rows.is_empty() {
        return false;
    }

    // 检查行数
    if result.rows.len() > MAX_RESULT_ROWS {
        return false;
    }

    // 检查大小
    let mut size = 0;
    for row in &result.rows {
        for val in row {
            size += val.estimate_memory_size();
            if size > MAX_RESULT_SIZE_BYTES {
                return false;
            }
        }
    }

    true
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/query_cache.rs
git commit -m "feat: add should_cache strategy"
```

---

### Task 2.3: 集成到 LocalExecutor

**Files:**
- Modify: `crates/executor/src/local_executor.rs`

**Step 1: 添加缓存字段**

```rust
use crate::query_cache::{CacheKey, QueryCache, QueryCacheConfig};
use crate::sql_normalizer::SqlNormalizer;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct LocalExecutor<'a> {
    storage: &'a dyn StorageEngine,
    cache: Arc<RwLock<QueryCache>>,
    cache_config: QueryCacheConfig,
}

impl<'a> LocalExecutor<'a> {
    pub fn new(storage: &'a dyn StorageEngine) -> Self {
        Self {
            storage,
            cache: Arc::new(RwLock::new(QueryCache::new(QueryCacheConfig::default()))),
            cache_config: QueryCacheConfig::default(),
        }
    }

    pub fn with_cache_config(storage: &'a dyn StorageEngine, config: QueryCacheConfig) -> Self {
        Self {
            storage,
            cache: Arc::new(RwLock::new(QueryCache::new(config.clone()))),
            cache_config: config,
        }
    }

    pub fn invalidate_table(&self, table: &str) {
        self.cache.write().invalidate_table(table);
    }

    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    fn get_cache_key(&self, sql: &str, params: &[Value]) -> CacheKey {
        let (normalized, extracted) = SqlNormalizer::from_literal(sql);
        let mut all_params = extracted;
        all_params.extend_from_slice(params);
        let hash = SqlNormalizer::hash_params(&all_params);
        CacheKey {
            normalized_sql: normalized,
            params_hash: hash,
        }
    }
}
```

**Step 2: 修改 execute 方法集成缓存**

```rust
impl<'a> Executor for LocalExecutor<'a> {
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let table_name = plan.table_name().to_string();
        
        // 尝试从缓存获取
        let cache_key = self.get_cache_key("", &[]);
        if let Some(result) = self.cache.read().get(&cache_key) {
            return Ok(result);
        }

        // 执行查询
        let result = LocalExecutor::execute(self, plan)?;

        // 判断是否缓存
        if should_cache(&result) && !table_name.is_empty() {
            let entry = CacheEntry {
                result: result.clone(),
                tables: vec![table_name],
                created_at: std::time::Instant::now(),
                size_bytes: 0, // 会在 put 中计算
            };
            self.cache.write().put(cache_key, entry, vec![table_name]);
        }

        Ok(result)
    }
}
```

**Step 3: 提交**

```bash
git add crates/executor/src/local_executor.rs
git commit -m "feat: integrate query cache into LocalExecutor"
```

---

## Day 3: 失效 + Metrics + 测试

### Task 3.1: 集成 Metrics

**Files:**
- Create: `crates/executor/src/query_cache_metrics.rs`

**Step 1: 创建 QueryCacheMetrics**

```rust
use sqlrustgo_common::metrics::{MetricValue, Metrics};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct QueryCacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    invalidations: AtomicU64,
}

impl QueryCacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_invalidation(&self, count: usize) {
        self.invalidations.fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl Default for QueryCacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics for QueryCacheMetrics {
    fn record_cache_hit(&mut self) {
        self.record_hit();
    }

    fn record_cache_miss(&mut self) {
        self.record_miss();
    }

    fn record_query(&mut self, _: &str, _: u64) {}

    fn record_error(&mut self) {}

    fn record_error_with_type(&mut self, _: &str) {}

    fn record_bytes_read(&mut self, _: u64) {}

    fn record_bytes_written(&mut self, _: u64) {}

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "query_cache_hits" => Some(MetricValue::Counter(self.hits.load(Ordering::Relaxed))),
            "query_cache_misses" => Some(MetricValue::Counter(self.misses.load(Ordering::Relaxed))),
            "query_cache_evictions" => Some(MetricValue::Counter(self.evictions.load(Ordering::Relaxed))),
            "query_cache_invalidations" => Some(MetricValue::Counter(self.invalidations.load(Ordering::Relaxed))),
            "query_cache_hit_rate" => Some(MetricValue::Gauge(self.hit_rate())),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "query_cache_hits".to_string(),
            "query_cache_misses".to_string(),
            "query_cache_evictions".to_string(),
            "query_cache_invalidations".to_string(),
            "query_cache_hit_rate".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.invalidations.store(0, Ordering::Relaxed);
    }
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/query_cache_metrics.rs
git commit -m "feat: add QueryCacheMetrics"
```

---

### Task 3.2: Storage 层触发失效

**Files:**
- Modify: `crates/storage/src/engine.rs`

**Step 1: 添加缓存失效回调**

```rust
pub trait StorageEngine: Send + Sync {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>>;
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()>;
    fn delete(&mut self, table: &str, filters: &[Value]) -> SqlResult<usize>;
    fn update(&mut self, table: &str, filters: &[Value], updates: &[(usize, Value)]) -> SqlResult<usize>;
    // ... existing methods
    
    fn on_write_complete(&mut self, table: &str) {
        // 回调，由上层实现触发缓存失效
    }
}
```

**Step 2: 在 MemoryStorage 中实现**

```rust
impl StorageEngine for MemoryStorage {
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        // ... 现有逻辑
        self.on_write_complete(table);
        Ok(())
    }
    
    fn on_write_complete(&mut self, table: &str) {
        // 这个由 LocalExecutor 设置的回调
        if let Some(callback) = &self.write_callback {
            callback(table);
        }
    }
}
```

**Step 3: 提交**

```bash
git add crates/storage/src/engine.rs
git commit -m "feat: add storage write callback for cache invalidation"
```

---

### Task 3.3: 单元测试

**Files:**
- Create: `crates/executor/tests/query_cache_test.rs`

**Step 1: 编写测试**

```rust
use sqlrustgo_executor::{QueryCache, QueryCacheConfig, CacheKey, CacheEntry, ExecutorResult};
use sqlrustgo_types::Value;
use std::time::Instant;

#[test]
fn test_cache_basic_get_put() {
    let config = QueryCacheConfig {
        max_entries: 10,
        max_memory_bytes: 1024 * 1024,
        ttl_seconds: 60,
        enabled: true,
    };
    let mut cache = QueryCache::new(config);

    let key = CacheKey {
        normalized_sql: "select * from t".to_string(),
        params_hash: 123,
    };
    let entry = CacheEntry {
        result: ExecutorResult::new(vec![vec![Value::Integer(1)]], 1),
        tables: vec!["t".to_string()],
        created_at: Instant::now(),
        size_bytes: 100,
    };

    cache.put(key.clone(), entry.clone(), vec!["t".to_string()]);
    
    let result = cache.get(&key);
    assert!(result.is_some());
    assert_eq!(result.unwrap().rows.len(), 1);
}

#[test]
fn test_cache_invalidate_table() {
    let mut cache = QueryCache::new(QueryCacheConfig::default());
    
    let key1 = CacheKey {
        normalized_sql: "select * from t1".to_string(),
        params_hash: 1,
    };
    cache.put(key1, CacheEntry::dummy(), vec!["t1".to_string()]);

    let key2 = CacheKey {
        normalized_sql: "select * from t2".to_string(),
        params_hash: 2,
    };
    cache.put(key2, CacheEntry::dummy(), vec!["t2".to_string()]);

    cache.invalidate_table("t1");
    
    assert!(cache.get(&CacheKey { normalized_sql: "select * from t1".to_string(), params_hash: 1 }).is_none());
    assert!(cache.get(&CacheKey { normalized_sql: "select * from t2".to_string(), params_hash: 2 }).is_some());
}

#[test]
fn test_cache_lru_eviction() {
    let mut config = QueryCacheConfig {
        max_entries: 2,
        ..Default::default()
    };
    let mut cache = QueryCache::new(config);

    cache.put(
        CacheKey { normalized_sql: "q1".to_string(), params_hash: 1 },
        CacheEntry::dummy(),
        vec!["t".to_string()],
    );
    cache.put(
        CacheKey { normalized_sql: "q2".to_string(), params_hash: 2 },
        CacheEntry::dummy(),
        vec!["t".to_string()],
    );
    cache.put(
        CacheKey { normalized_sql: "q3".to_string(), params_hash: 3 },
        CacheEntry::dummy(),
        vec!["t".to_string()],
    );

    assert!(cache.get(&CacheKey { normalized_sql: "q1".to_string(), params_hash: 1 }).is_none());
    assert!(cache.get(&CacheKey { normalized_sql: "q2".to_string(), params_hash: 2 }).is_some());
    assert!(cache.get(&CacheKey { normalized_sql: "q3".to_string(), params_hash: 3 }).is_some());
}

impl CacheEntry {
    fn dummy() -> Self {
        Self {
            result: ExecutorResult::empty(),
            tables: vec![],
            created_at: Instant::now(),
            size_bytes: 10,
        }
    }
}
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-executor query_cache
```

**Step 3: 提交**

```bash
git add crates/executor/tests/query_cache_test.rs
git commit -m "test: add query cache unit tests"
```

---

### Task 3.4: 创建 PR

**Step 1: 推送到远程**

```bash
git push -u origin feat/P-01-query-cache
```

**Step 2: 创建 PR**

```bash
gh pr create \
  --title "feat: P-01 查询缓存" \
  --body "$(cat <<'EOF'
## Summary
- 实现查询结果缓存 (Query Cache)
- 支持 LRU 淘汰策略
- 支持表级失效机制
- 支持 TTL (默认 30s)
- 集成 Metrics 统计

## Features
- CacheKey = normalized_sql + params_hash
- 独立 LRU 实现，不复用 BufferPool
- should_cache 策略过滤大结果集

## Metrics
- query_cache_hits
- query_cache_misses
- query_cache_evictions
- query_cache_invalidations
EOF
)" \
  --base develop/v1.6.0
```

**Step 3: 更新 Issue #601**

---

## 验证命令

```bash
# 运行所有测试
cargo test -p sqlrustgo-executor

# 运行缓存相关测试
cargo test -p sqlrustgo-executor query_cache

# 检查代码格式
cargo fmt --check

# 运行 clippy
cargo clippy -p sqlrustgo-executor -- -D warnings
```
