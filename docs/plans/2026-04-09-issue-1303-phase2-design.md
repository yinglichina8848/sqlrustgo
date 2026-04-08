# Issue #1303 Phase 2: 高级索引选择 - 设计文档

> **Date**: 2026-04-09
> **Author**: Claude Code
> **Status**: Ready for Implementation
> **Phase**: Phase 2 (Task 1 → 3 → 2 → 4)

---

## 概述

Phase 1 已实现基础索引扫描（等值 + 范围查询）。Phase 2 目标：

1. **Task 1**: 成本模型集成 - CboOptimizer 与 IndexSelect 深度集成
2. **Task 3**: 多列索引 - CompositeBTreeIndex 完整功能
3. **Task 2**: ANALYZE 自动同步 - 统计信息自动传播
4. **Task 4**: 索引 Hint - MySQL 风格 Hint 语法

---

## Task 1: 成本模型集成

### 1.1 架构变更：OptimizerContext

**问题**: 未来会有更多规则需要 stats_provider 和 cost_model，散点注入会导致 API 膨胀。

**解决方案**: 引入 `OptimizerContext` 统一上下文：

```rust
// optimizer/src/context.rs

/// Optimizer 全局上下文 - 持有成本模型和统计信息提供者
#[derive(Clone)]
pub struct OptimizerContext {
    /// 统计信息提供者
    pub stats_provider: Arc<dyn StatisticsProvider>,
    /// 成本模型
    pub cost_model: Arc<SimpleCostModel>,
}

impl OptimizerContext {
    pub fn new(stats_provider: Arc<dyn StatisticsProvider>, cost_model: SimpleCostModel) -> Self {
        Self {
            stats_provider,
            cost_model: Arc::new(cost_model),
        }
    }

    pub fn with_default_stats_provider(self) -> Self {
        if self.stats_provider.as_ref().as_any().downcast_ref::<InMemoryStatisticsProvider>().is_none() {
            // 使用默认的内存统计提供者
            self
        } else {
            self
        }
    }
}

impl Default for OptimizerContext {
    fn default() -> Self {
        Self {
            stats_provider: Arc::new(InMemoryStatisticsProvider::new()),
            cost_model: Arc::new(SimpleCostModel::default_model()),
        }
    }
}
```

### 1.2 IndexSelect 规则增强

**变更前**:
```rust
pub struct IndexSelect {
    cost_model: SimpleCostModel,
    available_indexes: Vec<(String, String)>,
}
```

**变更后**:
```rust
pub struct IndexSelect {
    optimizer_ctx: Arc<OptimizerContext>,  // 统一上下文
    available_indexes: Vec<(String, String)>,
}

impl IndexSelect {
    pub fn new(optimizer_ctx: Arc<OptimizerContext>) -> Self {
        Self {
            optimizer_ctx,
            available_indexes: Vec::new(),
        }
    }

    pub fn with_index(mut self, table: impl Into<String>, index: impl Into<String>) -> Self {
        self.available_indexes.push((table.into(), index.into()));
        self
    }

    /// 基于成本的索引选择（纯 CBO，无 heuristic threshold）
    fn should_use_index(&self, table: &str, column: &str) -> IndexAccessMethod {
        // 1. 检查是否有可用索引
        if !self.available_indexes.iter().any(|(t, _)| t == table) {
            return IndexAccessMethod::SeqScan;
        }

        // 2. 估算顺序扫描成本
        let seq_scan_cost = self.optimizer_ctx.cost_model.estimate_scan_cost(
            &self.optimizer_ctx.stats_provider,
            table,
        );

        // 3. 估算索引扫描成本
        let index_scan_cost = self.optimizer_ctx.cost_model.estimate_index_scan_cost(
            &self.optimizer_ctx.stats_provider,
            table,
            column,
        );

        // 4. 纯成本决策：无 threshold，纯比较
        if index_scan_cost < seq_scan_cost {
            IndexAccessMethod::IndexScan
        } else {
            IndexAccessMethod::SeqScan
        }
    }
}
```

### 1.3 关键修正：删除 selectivity_threshold

**错误逻辑**:
```rust
// ❌ 错误：混合 heuristic 和 CBO
return selectivity < threshold && index_cost < seq_cost;
```

**正确逻辑**:
```rust
// ✅ 正确：纯 CBO 成本决策
return index_cost < seq_cost;
```

### 1.4 文件变更

| 文件 | 变更 |
|------|------|
| `optimizer/src/context.rs` | 新增 OptimizerContext |
| `optimizer/src/lib.rs` | 导出 OptimizerContext |
| `optimizer/src/rules.rs` | IndexSelect 使用 OptimizerContext |

### 1.5 验证标准

- [ ] IndexSelect 通过 OptimizerContext 获取统计信息
- [ ] 成本估算使用真实统计信息（非默认）
- [ ] 无 selectivity_threshold heuristic

---

## Task 2: 多列索引

### 2.1 CompositeKey 结构修正

**问题**: 旧设计将多列编码为单个 i64，会产生冲突。

**正确设计**:
```rust
// storage/src/bplus_tree/index.rs

/// 复合索引键 - 使用 Vec<Value> 而非 i64
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositeKey {
    pub values: Vec<Value>,
}

impl CompositeKey {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn from_row(row: &[Value], column_indices: &[usize]) -> Self {
        Self {
            values: column_indices
                .iter()
                .map(|&i| row.get(i).cloned().unwrap_or(Value::Null))
                .collect(),
        }
    }
}

/// 字典序比较器
impl Ord for CompositeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        for (lhs, rhs) in self.values.iter().zip(other.values.iter()) {
            match lhs.partial_cmp(rhs) {
                Some(Ordering::Equal) => continue,
                Some(cmp) => return cmp,
                None => return Ordering::Equal,
            }
        }
        self.values.len().cmp(&other.values.len())
    }
}

impl PartialOrd for CompositeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
```

### 2.2 StorageEngine API 修正

**问题**: 使用 `table + index_name` string lookup 效率低。

**正确设计**: 使用 `IndexId(u32)` 统一管理：

```rust
// storage/src/engine.rs

/// 索引唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexId(pub u32);

/// StorageEngine trait 扩展
trait StorageEngine {
    // ... existing methods ...

    /// 创建复合索引
    fn create_composite_index(
        &mut self,
        table: &str,
        columns: Vec<String>,
    ) -> SqlResult<IndexId>;

    /// 复合索引等值查询
    fn search_composite_index(
        &self,
        index_id: IndexId,
        key: &CompositeKey,
    ) -> SqlResult<Vec<u32>>;

    /// 复合索引范围查询
    fn range_composite_index(
        &self,
        index_id: IndexId,
        start: &CompositeKey,
        end: &CompositeKey,
    ) -> SqlResult<Vec<u32>>;
}
```

### 2.3 索引注册表

```rust
// storage/src/index_registry.rs

/// 索引注册表 - 管理所有索引元数据
pub struct IndexRegistry {
    indexes: HashMap<IndexId, IndexMeta>,
    name_to_id: HashMap<String, IndexId>,
}

#[derive(Debug, Clone)]
pub struct IndexMeta {
    pub id: IndexId,
    pub table_name: String,
    pub column_names: Vec<String>,
    pub index_type: IndexType,  // BTree, Hash, Composite
}

enum IndexType {
    BTree,
    Hash,
    CompositeBTree,
}
```

### 2.4 文件变更

| 文件 | 变更 |
|------|------|
| `storage/src/bplus_tree/index.rs` | CompositeKey + comparator |
| `storage/src/engine.rs` | 扩展 StorageEngine trait |
| `storage/src/index_registry.rs` | 新增索引注册表 |
| `executor/src/local_executor.rs` | 支持复合索引执行 |

### 2.5 验证标准

- [ ] `CREATE INDEX idx ON users(id, name)` 成功创建复合索引
- [ ] `WHERE id = 1 AND name = 'Alice'` 使用复合索引
- [ ] 复合索引范围查询正常工作

---

## Task 3: ANALYZE 自动同步

### 3.1 StatsRegistry 架构

**问题**: storage 直接更新 optimizer stats 会造成反向依赖。

**解决方案**: 引入 StatsRegistry 回调机制：

```rust
// optimizer/src/stats_registry.rs

/// 统计信息注册表 - 管理全局统计信息
pub struct StatsRegistry {
    provider: Arc<dyn StatisticsProvider>,
}

impl StatsRegistry {
    pub fn new(provider: Arc<dyn StatisticsProvider>) -> Self {
        Self { provider }
    }

    /// 同步表的统计信息（由 storage ANALYZE 触发后调用）
    pub fn sync_table_stats(&self, table: &str, storage_stats: StorageTableStats) -> SqlResult<()> {
        let optimizer_stats: TableStats = storage_stats.into();
        self.provider.update_stats(table, optimizer_stats)?;
        Ok(())
    }

    /// 获取统计信息提供者
    pub fn provider(&self) -> Arc<dyn StatisticsProvider> {
        self.provider.clone()
    }
}

impl StatisticsProvider for StatsRegistry {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        self.provider.table_stats(table)
    }

    fn update_stats(&self, table: &str, stats: TableStats) -> StatsResult<()> {
        self.provider.update_stats(table, stats)
    }

    fn selectivity(&self, table: &str, column: &str) -> f64 {
        self.provider.selectivity(table, column)
    }
}
```

### 3.2 ANALYZE 执行流程

```
SQL: ANALYZE users
    ↓
Parser: AnalyzeStatement { table_name: Some("users") }
    ↓
Executor: execute_analyze(storage, "users")
    ↓
Storage: storage.analyze_table("users") → StorageTableStats
    ↓
StatsRegistry: sync_table_stats("users", storage_stats)
    ↓
InMemoryStatisticsProvider: update_stats("users", optimizer_stats)
    ↓
OptimizerContext: 可用新统计信息
```

### 3.3 Stale Statistics 风险处理

```rust
// optimizer/src/stats.rs

#[derive(Debug, Clone)]
pub struct TableStats {
    // ... existing fields ...
    
    /// 统计信息版本号
    pub version: u64,
    /// 统计信息时间戳
    pub last_updated: DateTime<Utc>,
}

impl TableStats {
    pub fn is_stale(&self, threshold: Duration) -> bool {
        let age = Utc::now() - self.last_updated;
        age > threshold
    }
}
```

### 3.4 文件变更

| 文件 | 变更 |
|------|------|
| `optimizer/src/stats_registry.rs` | 新增 StatsRegistry |
| `optimizer/src/stats.rs` | 添加 version/last_updated |
| `src/lib.rs` | ANALYZE 后触发 sync |
| `optimizer/src/lib.rs` | 导出 StatsRegistry |

### 3.5 验证标准

- [ ] `ANALYZE users` 后 optimizer 可读取新统计信息
- [ ] 多次 ANALYZE 版本号递增
- [ ] StatsRegistry 正确解耦 storage 和 optimizer

---

## Task 4: 索引 Hint

### 4.1 AST 变更

**问题**: Vec<IndexHint> 无法处理多表不同 hint。

**正确设计**: `HashMap<TableAlias, IndexHint>`:

```rust
// parser/src/parser.rs

/// 索引 Hint 类型
#[derive(Debug, Clone, PartialEq)]
pub enum IndexHintType {
    /// USE INDEX - 优先使用指定索引
    UseIndex,
    /// FORCE INDEX - 强制使用指定索引
    ForceIndex,
    /// IGNORE INDEX - 禁止使用指定索引
    IgnoreIndex,
}

/// 索引 Hint
#[derive(Debug, Clone, PartialEq)]
pub struct IndexHint {
    /// 关联的表别名（None 表示主表）
    pub table_alias: Option<String>,
    /// Hint 类型
    pub hint_type: IndexHintType,
    /// 索引名列表
    pub index_names: Vec<String>,
}

/// SelectStatement 扩展
pub struct SelectStatement {
    // ... existing fields ...
    
    /// 每表的索引 Hint（key: 表别名）
    pub index_hints: HashMap<String, IndexHint>,
}
```

### 4.2 Hint 优先级

**关键规则**（必须严格执行）:

```rust
// Hint 优先级（从高到低）
// 1. FORCE INDEX > 2. IGNORE INDEX > 3. USE INDEX > 4. CBO Decision

impl IndexHint {
    pub fn effective_index(&self, available_indexes: &[String]) -> Option<String> {
        match self.hint_type {
            IndexHintType::ForceIndex => {
                // 强制使用第一个可用索引（忽略 CBO）
                self.index_names.first().cloned()
            }
            IndexHintType::IgnoreIndex => {
                // 返回第一个非 ignore 的索引
                available_indexes
                    .iter()
                    .find(|idx| !self.index_names.contains(idx))
                    .cloned()
            }
            IndexHintType::UseIndex => {
                // 优先使用 hint 索引，但 CBO 可覆盖
                self.index_names.first().cloned()
            }
        }
    }
}
```

### 4.3 解析语法

```sql
-- MySQL 风格
SELECT * FROM users USE INDEX (idx_name) WHERE id = 1
SELECT * FROM users FORCE INDEX (idx_pkey) WHERE id = 1
SELECT * FROM users IGNORE INDEX (idx_scan) WHERE id = 1

-- 多表
SELECT * FROM users u USE INDEX (idx_age), orders o IGNORE INDEX (idx_user)
```

### 4.4 Planner → Optimizer 传递

```rust
// planner/src/planner.rs

impl Planner {
    fn create_physical_plan(&self, logical: &LogicalPlan) -> PhysicalPlan {
        // 从 SelectStatement 获取 hints
        let hints = &self.select_stmt.index_hints;
        
        // 传递给 optimizer
        let mut index_select = IndexSelect::new(self.optimizer_ctx.clone());
        for (table_alias, hint) in hints {
            if let Some(idx) = hint.effective_index(&available_indexes) {
                index_select.add_hint(table_alias, idx);
            }
        }
        
        // ...
    }
}
```

### 4.5 文件变更

| 文件 | 变更 |
|------|------|
| `parser/src/parser.rs` | IndexHint AST + 解析 |
| `parser/src/token.rs` | 添加 USE, FORCE, IGNORE token |
| `planner/src/planner.rs` | 将 hints 传递到 optimizer |
| `optimizer/src/rules.rs` | IndexSelect 处理 hints |

### 4.6 验证标准

- [ ] `SELECT * FROM users USE INDEX (idx) WHERE id = 1` 解析成功
- [ ] `SELECT * FROM users FORCE INDEX (idx) WHERE id = 1` 强制使用索引
- [ ] `SELECT * FROM users IGNORE INDEX (idx) WHERE id = 1` 跳过索引
- [ ] 多表不同 hint 正确处理

---

## 风险分析

| 风险 | 影响 | 缓解 |
|------|------|------|
| **Stale statistics** | INSERT 后未 ANALYZE 导致错误执行计划 | 添加 version/timestamp，支持 auto-analyze |
| **CompositeKey 冲突** | 编码冲突导致查询结果错误 | 使用 Vec<Value> + comparator，无编码 |
| **Hint vs CBO 冲突** | Hint 可能产生更差的执行计划 | FORCE > IGNORE > USE > CBO 优先级明确 |
| **依赖方向倒置** | storage 依赖 optimizer | StatsRegistry 回调机制 |

---

## 实现顺序

```
Phase 2 (Task 1 → 3 → 2 → 4)

Task 1: OptimizerContext + IndexSelect CBO 集成
    ↓
Task 3: CompositeKey + 多列索引
    ↓
Task 2: StatsRegistry + ANALYZE 同步
    ↓
Task 4: Index Hint (MySQL 风格)
```

---

## 验收清单

- [ ] IndexSelect 通过 OptimizerContext 获取统计信息
- [ ] 成本决策使用纯 CBO（无 heuristic threshold）
- [ ] CompositeKey 使用 Vec<Value> + lexicographic comparator
- [ ] StorageEngine 使用 IndexId 而非 string lookup
- [ ] StatsRegistry 正确解耦 storage 和 optimizer
- [ ] ANALYZE 自动同步统计信息到 optimizer
- [ ] 索引 Hint 支持 per-table HashMap 结构和优先级
- [ ] Stats staleness 有 version/timestamp 处理
