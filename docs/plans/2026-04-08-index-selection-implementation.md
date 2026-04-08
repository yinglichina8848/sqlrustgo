# Issue #1303 - 查询计划器 - 自动选择索引

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 根据统计信息自动选择最优索引，实现基于成本的索引选择器 (CBO-based Index Selection)。

**Architecture:** 增强现有的 CboOptimizer 和 IndexSelect 规则，集成统计信息收集和成本估算。

**Tech Stack:** Rust, crates/optimizer, crates/storage

---

## 现状分析

### 已有的基础设施

1. **CboOptimizer** (`crates/optimizer/src/cost.rs:270`)
   - `select_access_method()` 方法选择 index_scan 或 seq_scan
   - 需要 statistics provider 来获取真实统计信息

2. **SimpleCostModel** (`crates/optimizer/src/cost.rs`)
   - `seq_scan_cost()` - 顺序扫描成本
   - `index_scan_cost()` - 索引扫描成本
   - `estimate_index_scan_with_stats()` - 使用统计信息的索引扫描成本估算

3. **StatisticsProvider trait** (`crates/optimizer/src/stats.rs:189`)
   - `table_stats()` - 获取表统计信息
   - `selectivity()` - 获取列选择性
   - `InMemoryStatisticsProvider` - 内存实现

4. **StatsCollector trait** (`crates/optimizer/src/stats.rs:271`)
   - `collect_table_stats()` - 收集表级统计
   - `collect_column_stats()` - 收集列级统计
   - `DefaultStatsCollector` - 默认实现

5. **IndexSelect rule** (`crates/optimizer/src/rules.rs:856`)
   - 当前只是启发式方法，没有真正使用成本估算
   - `should_use_index()` 只检查是否有可用索引和谓词是否可索引化

### 缺失的部分

1. **统计信息自动收集** - 统计数据不会自动更新
2. **StatisticsProvider 持久化** - 只有内存实现
3. **成本模型集成** - IndexSelect 没有使用 CboOptimizer 的成本估算
4. **索引可用性发现** - IndexSelect 需要知道哪些索引实际存在

---

## 实现计划

### Phase 1: 增强统计信息收集

#### Task 1: 添加 ANALYZE TABLE 命令支持

**Files:**
- Modify: `crates/planner/src/planner.rs` - 添加 ANALYZE 命令解析
- Modify: `crates/executor/src/` - 添加 ANALYZE 执行器

**Step 1: 在 Planner 中添加 ANALYZE 命令处理**

在 `planner.rs` 中添加：

```rust
pub enum Statement {
    // ... existing variants
    Analyze {
        table_name: String,
    },
}
```

**Step 2: 在 Executor 中实现 ANALYZE**

```rust
pub struct AnalyzeExecutor {
    storage: Arc<dyn StorageEngine>,
    stats_collector: DefaultStatsCollector,
}

impl AnalyzeExecutor {
    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage,
            stats_collector: DefaultStatsCollector::new(),
        }
    }

    pub fn execute(&self, table: &str) -> Result<TableStats> {
        self.stats_collector.collect_table_stats(self.storage.as_ref(), table)
    }
}
```

**Step 3: 验证编译**

```bash
cargo check -p sqlrustgo-planner -p sqlrustgo-executor
```

---

#### Task 2: 添加 StatisticsProvider 持久化支持

**Files:**
- Create: `crates/optimizer/src/stats_provider.rs`

**Step 1: 创建持久化 StatisticsProvider 接口**

```rust
/// Persistent statistics provider that saves stats to storage
pub trait PersistentStatisticsProvider: StatisticsProvider {
    /// Save statistics to persistent storage
    fn save_stats(&self, table: &str, stats: &TableStats) -> Result<()>;

    /// Load statistics from persistent storage
    fn load_stats(&self, table: &str) -> Result<Option<TableStats>>;
}

/// StatisticsProvider adapter that wraps another provider with caching
pub struct CachedStatisticsProvider {
    inner: Box<dyn StatisticsProvider>,
    cache: HashMap<String, TableStats>,
}
```

**Step 2: 实现缓存包装器**

```rust
impl CachedStatisticsProvider {
    pub fn new(inner: Box<dyn StatisticsProvider>) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }
}

impl StatisticsProvider for CachedStatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        self.cache.get(table).cloned().or_else(|| {
            let stats = self.inner.table_stats(table);
            if let Some(ref s) = stats {
                self.cache.insert(table.to_string(), s.clone());
            }
            stats
        })
    }
}
```

**Step 3: 验证编译**

---

### Phase 2: 增强成本模型

#### Task 3: 增强 CboOptimizer 的索引选择逻辑

**Files:**
- Modify: `crates/optimizer/src/cost.rs`

**Step 1: 添加更精确的成本估算方法**

```rust
impl CboOptimizer {
    /// Estimate cost for range scan using index
    pub fn estimate_range_scan_cost(
        &self,
        table: &str,
        column: &str,
        range selectivity: f64,
    ) -> f64 {
        if let Some(ref provider) = self.stats_provider {
            if let Some(stats) = provider.table_stats(table) {
                let row_count = stats.row_count();
                let page_count = stats.page_count();
                let index_pages = ((row_count as f64 * selectivity) / 100.0).max(1.0) as u64;

                return self.cost_model.index_scan_cost(
                    (row_count as f64 * selectivity) as u64,
                    index_pages,
                    page_count,
                );
            }
        }
        // Default estimate
        self.cost_model.index_scan_cost(100, 1, 10)
    }

    /// Select best access method considering all factors
    pub fn select_best_access_method(
        &self,
        table: &str,
        column: &str,
        predicate_type: PredicateType,
    ) -> AccessMethod {
        let selectivity = self
            .stats_provider
            .as_ref()
            .and_then(|p| p.selectivity(table, column).into());

        match predicate_type {
            PredicateType::Eq => self.select_access_method(table, column, 0.05),
            PredicateType::Range => {
                let seq_cost = self.estimate_scan_cost(table);
                let range_cost = self.estimate_range_scan_cost(table, column, selectivity);
                if range_cost < seq_cost {
                    AccessMethod::IndexScan
                } else {
                    AccessMethod::SeqScan
                }
            }
            PredicateType::In => AccessMethod::IndexScan,
            PredicateType::Like => AccessMethod::SeqScan, // Can't use index for LIKE
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    Eq,
    Range,
    In,
    Like,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMethod {
    SeqScan,
    IndexScan,
    HashIndex,
}
```

**Step 2: 添加 HashIndex 成本估算**

```rust
impl SimpleCostModel {
    /// Estimate cost for hash index lookup
    pub fn hash_index_cost(&self, row_count: u64, index_pages: u64) -> f64 {
        // Hash index: O(1) lookup, but still has I/O cost
        let index_cost = index_pages as f64 * self.io_cost_per_page;
        let cpu_cost = row_count as f64 * self.cpu_cost_per_row * 0.1; // Very cheap lookup
        index_cost + cpu_cost
    }
}
```

---

#### Task 4: 增强 IndexSelect 规则使用成本模型

**Files:**
- Modify: `crates/optimizer/src/rules.rs`

**Step 1: 修改 IndexSelect 结构体**

```rust
pub struct IndexSelect {
    cost_model: CboOptimizer,
    available_indexes: Vec<IndexInfo>,
    stats_provider: Option<Box<dyn StatisticsProvider>>,
}

struct IndexInfo {
    table_name: String,
    column_name: String,
    index_type: IndexType,
}

enum IndexType {
    BTree,
    Hash,
}
```

**Step 2: 添加 from_storage 方法**

```rust
impl IndexSelect {
    /// Create IndexSelect from storage engine
    pub fn from_storage(storage: &dyn StorageEngine) -> Self {
        let mut selector = Self::new();

        // Discover available indexes from storage
        for table_name in storage.list_tables() {
            if let Ok(table_info) = storage.get_table_info(&table_name) {
                // Check which columns have indexes
                // This would need storage API to expose index information
            }
        }

        selector
    }

    /// Add a stats provider for cost estimation
    pub fn with_stats_provider(mut self, provider: Box<dyn StatisticsProvider>) -> Self {
        self.stats_provider = Some(provider);
        self
    }
}
```

**Step 3: 修改 should_use_index 方法使用成本估算**

```rust
impl IndexSelect {
    fn should_use_index(&self, table: &str, predicate: &Expr) -> bool {
        // First check if predicate is indexable
        if !self.is_indexable_predicate(predicate) {
            return false;
        }

        // Get column from predicate
        let column = self.extract_column_name(predicate);

        // Use cost model to decide
        if let Some(ref provider) = self.stats_provider {
            let selectivity = provider.selectivity(table, &column);
            let predicate_type = self.classify_predicate(predicate);

            let access_method = self.cost_model.select_best_access_method(
                table,
                &column,
                predicate_type,
            );

            return access_method == AccessMethod::IndexScan;
        }

        // Fallback: use index if available
        self.available_indexes.iter().any(|(t, _)| t == table)
    }
}
```

---

### Phase 3: 集成和测试

#### Task 5: 集成到系统

**Files:**
- Modify: `crates/planner/src/planner.rs` - 添加 OPTIMIZE TABLE 命令
- Modify: `crates/executor/src/executor.rs` - 添加统计信息自动更新

**Step 1: 添加 OPTIMIZE TABLE 支持**

```rust
pub enum Statement {
    Optimize {
        table_name: String,
        // Options: ANALYZE, RECLAIM, etc.
    },
}
```

**Step 2: 集成到默认优化器**

```rust
// In optimizer initialization
let stats_provider = Arc::new(InMemoryStatisticsProvider::new());
let cbo_optimizer = CboOptimizer::new()
    .with_stats_provider(Box::new(stats_provider.clone()));

let index_select = IndexSelect::new()
    .with_stats_provider(Box::new(stats_provider));
```

---

#### Task 6: 添加测试

**Files:**
- Create: `crates/optimizer/tests/index_selection_test.rs`

**Step 1: 测试成本估算**

```rust
#[test]
fn test_index_selection_low_selectivity() {
    let mut provider = InMemoryStatisticsProvider::new();
    provider.add_stats(
        TableStats::new("users")
            .with_row_count(10000)
            .add_column_stats(
                ColumnStats::new("id")
                    .with_distinct_count(10000) // High cardinality
            )
    );

    let cbo = CboOptimizer::new()
        .with_stats_provider(Box::new(provider));

    // For high selectivity (low selectivity = few rows match), use index
    let method = cbo.select_access_method("users", "id", 0.0001);
    assert_eq!(method, "index_scan");
}

#[test]
fn test_index_selection_high_selectivity() {
    let mut provider = InMemoryStatisticsProvider::new();
    provider.add_stats(
        TableStats::new("users")
            .with_row_count(1000)
            .add_column_stats(
                ColumnStats::new("status")
                    .with_distinct_count(2) // Low cardinality
            )
    );

    let cbo = CboOptimizer::new()
        .with_stats_provider(Box::new(provider));

    // For low selectivity (high selectivity = many rows match), use seq scan
    let method = cbo.select_access_method("users", "status", 0.5);
    assert_eq!(method, "seq_scan");
}
```

---

## 验收标准

- [ ] Task 1: ANALYZE TABLE 命令成功执行并收集统计信息
- [ ] Task 2: 统计信息可以持久化和加载
- [ ] Task 3: CboOptimizer 正确估算 seq_scan 和 index_scan 成本
- [ ] Task 4: IndexSelect 规则使用成本模型选择索引
- [ ] Task 5: 系统可以自动收集和使用统计信息
- [ ] Task 6: 单元测试和集成测试通过

## 预估工时

4-5 人天

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-04-08-index-selection-implementation.md`**

Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
