# SQLRustGo 大数据量性能优化设计文档

**日期**: 2026-04-12  
**状态**: 已批准  
**目标**: 全面提升 SQLRustGo 在大数据量场景（>10万行，1G-10G数据集）下的性能，使其达到或超过 SQLite 水平

---

## 1. 问题分析

### 1.1 性能数据

| 数据规模 | SQLRustGo | SQLite | 性能比 |
|----------|-----------|--------|--------|
| 1万行 | ~100ms | 15ms | 6.7x |
| 60万行 | 1.7s | 52ms | **33x** 差距 |
| 小数据(5行) | 1.34ms | 5.06ms | **3.8x** 优势 |

**核心问题**: SQLRustGo 在小数据量下比 SQLite 快，但大数据量下慢 33 倍。

### 1.2 根本原因

```
当前执行流程:
SQL → SeqScanExec → storage.scan() [返回全部60万行] → FilterExec [executor层过滤]
```

| 层级 | 问题 |
|------|------|
| **Storage** | `scan()` 返回全部行，无过滤 |
| **Executor** | `SeqScan` 不过滤谓词，`Filter` 在下游评估 |
| **Optimizer** | 代价模型未选择 IndexScan |

### 1.3 已有基础设施（未充分利用）

```
StorageEngine trait:
- search_index(&self, table, column, key) -> Vec<u32>
- range_index(&self, table, column, start, end) -> Vec<u32>

MemoryStorage 实现:
- indexes: HashMap<String, SimpleBPlusTree>
- hash_indexes: HashMap<String, Arc<HashIndex<i64, u32>>>
```

---

## 2. 架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SQL Query                                   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Query Optimizer                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  New Cost Model (with statistics)                            │   │
│  │  - TableStats: row_count, column_stats, index_info          │   │
│  │  - CostEstimator: scan_cost, index_cost                      │   │
│  │  - PathSelector: SeqScan vs IndexScan vs VectorScan         │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
         ┌──────────────────┐        ┌──────────────────┐
         │   SeqScanPath    │        │   IndexScanPath   │
         │  (full table)    │        │  (use index)      │
         └──────────────────┘        └──────────────────┘
                    │                           │
                    ▼                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Unified ScanExecutor Trait                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ trait ScanExecutor {                                         │   │
│  │   fn init(&mut self, storage: &dyn StorageEngine) -> ();     │   │
│  │   fn next(&mut self) -> Option<Record>;                    │   │
│  │   fn predicate(&self) -> Option<&Predicate>;                │   │
│  │   fn schema(&self) -> &Schema;                             │   │
│  │ }                                                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Storage Engine Layer                             │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐      │
│  │MemoryStorage│  │FileStorage  │  │   ColumnarStorage       │      │
│  │             │  │             │  │                         │      │
│  │ scan()      │  │ scan()      │  │ scan_columnar()         │      │
│  │ scan_batch()│  │ scan_batch()│  │ scan_columnar_batch()   │      │
│  │ scan_pred() │  │ scan_pred() │  │ scan_pred() [NEW]       │      │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘      │
│                                                                      │
│  Index Infrastructure:                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐      │
│  │  HashIndex  │  │  BPlusTree  │  │   CompositeIndex        │      │
│  │  O(1) eq   │  │  O(log n)   │  │   multi-column          │      │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. 核心组件设计

### 3.1 Predicate 结构（新增）

```rust
// crates/storage/src/predicate.rs (NEW)

pub enum Predicate {
    Eq(Box<Expr>, Box<Expr>),                    // a = b
    Lt(Box<Expr>, Box<Expr>),                    // a < b
    Lte(Box<Expr>, Box<Expr>),                   // a <= b
    Gt(Box<Expr>, Box<Expr>),                    // a > b
    Gte(Box<Expr>, Box<Expr>),                  // a >= b
    In(Box<Expr>, Vec<Expr>),                   // a IN (1,2,3)
    And(Box<Predicate>, Box<Predicate>),         // a AND b
    Or(Box<Predicate>, Box<Predicate>),          // a OR b
    Not(Box<Predicate>),                         // NOT a
    IsNull(Box<Expr>),                           // a IS NULL
    IsNotNull(Box<Expr>),                       // a IS NOT NULL
}

pub enum IndexOp {
    Eq(i64),                                     // column = value
    Range(i64, i64),                             // start <= column <= end
    Contains(String),                             // text contains
}
```

### 3.2 Storage Engine 新增接口

```rust
// crates/storage/src/engine.rs

/// 谓词下推扫描 - 存储层直接评估谓词，只返回匹配行
fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[ColumnIndex]>,
) -> Result<ScanResult, StorageError>;

/// 索引辅助扫描 - 返回满足条件的 row_ids
fn scan_with_index(
    &self,
    table: &str,
    column: &str,
    op: IndexOp,
    projection: Option<&[ColumnIndex]>,
) -> Result<ScanResult, StorageError>;
```

### 3.3 统计信息（新增）

```rust
// crates/optimizer/src/stats.rs (NEW)

#[derive(Clone)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub column_stats: HashMap<String, ColumnStats>,
    pub index_stats: HashMap<String, IndexStats>,
}

#[derive(Clone)]
pub struct ColumnStats {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
    pub histogram: Vec<f64>,
}

#[derive(Clone)]
pub struct IndexStats {
    pub index_name: String,
    pub index_type: IndexType,
    pub cardinality: u64,
    pub selectiveness: f64,  // 0.0-1.0，越高越适合用索引
}
```

### 3.4 代价模型（重构）

```rust
// crates/optimizer/src/cost.rs (重构)

pub struct CostModel {
    stats: StatsCache,
}

impl CostModel {
    pub fn estimate_seq_scan(&self, table: &TableStats) -> Cost {
        Cost {
            io_cost: table.row_count * ROW_SIZE / PAGE_SIZE,
            cpu_cost: table.row_count,
            memory_cost: 0,
        }
    }

    pub fn estimate_index_scan(&self, table: &TableStats, index: &IndexStats, op: &IndexOp) -> Cost {
        let selectivity = self.compute_selectivity(index, op);
        let rows_to_read = (table.row_count as f64 * selectivity) as u64;
        Cost {
            io_cost: rows_to_read * ROW_SIZE / PAGE_SIZE,
            cpu_cost: rows_to_read + INDEX_LOOKUP_COST,
            memory_cost: 0,
        }
    }
}
```

---

## 4. 实施阶段

### Phase 1: 谓词下推存储层

**目标**: 实现 `storage.scan_predicate()`，让存储层直接评估谓词

| 组件 | 文件 | 改动 |
|------|------|------|
| Predicate 定义 | `storage/src/predicate.rs` | 新增 |
| StorageEngine trait | `storage/src/engine.rs` | 新增 `scan_predicate` |
| MemoryStorage | `storage/src/engine.rs` | 实现 `scan_predicate` |
| FileStorage | `storage/src/file_storage.rs` | 实现 `scan_predicate` |
| ColumnarStorage | `storage/src/columnar/storage.rs` | 实现 `scan_predicate` |
| SeqScanExecutor | `executor/src/seq_scan.rs` | 接入 `scan_predicate` |
| FilterPushdown rule | `optimizer/src/rules.rs` | 新增规则 |

**关键实现**:

```rust
// MemoryStorage::scan_predicate 实现
fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[ColumnIndex]>,
) -> Result<ScanResult, StorageError> {
    // 1. 尝试使用索引
    if let Some((column, op)) = self.analyze_predicate_for_index(predicate) {
        if let Some(row_ids) = self.try_index_lookup(table, column, op)? {
            return self.fetch_rows_by_ids(table, row_ids, projection);
        }
    }

    // 2. 索引未命中，全表扫描 + 谓词过滤
    let all_rows = self.tables.get(table).ok_or(TableNotFound)?;
    let filtered: Vec<Record> = all_rows
        .iter()
        .filter(|row| self.eval_predicate(row, predicate))
        .cloned()
        .collect();

    // 3. Projection 下推
    self.apply_projection(filtered, projection)
}
```

### Phase 2: 索引优化 + 统一 ScanExecutor

**目标**: 统一 SeqScan/IndexScan 接口，优化器能正确选择索引路径

| 组件 | 文件 | 改动 |
|------|------|------|
| ScanExecutor trait | `executor/src/scan.rs` | 新增统一 trait |
| IndexScanExecutor | `executor/src/index_scan.rs` | 实现 ScanExecutor |
| SeqScanExecutor 重构 | `executor/src/seq_scan.rs` | 实现 ScanExecutor |
| IndexSelector rule | `optimizer/src/rules.rs` | 新增规则 |
| INDEX HINT 支持 | `planner/src/` | 支持 INDEX HINT 语法 |

### Phase 3: 新代价模型 + 统计信息收集

**目标**: 实现完整代价模型，支持统计信息收集

| 组件 | 文件 | 改动 |
|------|------|------|
| TableStats | `optimizer/src/stats.rs` | 新增 |
| StatsCollector | `optimizer/src/stats.rs` | 新增 |
| CostEstimator | `optimizer/src/cost.rs` | 重构 |
| PathSelector | `optimizer/src/path_selector.rs` | 重构 |
| ANALYZE 命令 | `sql/src/analyze.rs` | 新增 |

```sql
-- 新增 ANALYZE 命令用于收集统计信息
ANALYZE TABLE users;
```

---

## 5. 错误处理

| 场景 | 处理方式 |
|------|----------|
| 索引不存在 | 回退到 SeqScan |
| 谓词无法下推 | 在 Executor 层过滤 |
| 统计信息过期 | 使用启发式估算 + 定期 ANALYZE |
| 存储层返回错误 | 转换为 ExecutorError 传播 |
| NULL 值处理 | IS NULL / IS NOT NULL 统一处理 |

---

## 6. 测试策略

| 阶段 | 测试类型 | 覆盖 |
|------|----------|------|
| Phase 1 | 谓词单元测试 | 6 种比较操作 + AND/OR/NOT |
| Phase 1 | 集成测试 | TPC-H Q1-Q22 |
| Phase 1 | 性能基准 | SF0.1, 0.3, 1 对比 SQLite |
| Phase 2 | 索引选择测试 | 验证优化器选择正确路径 |
| Phase 2 | Hint 测试 | INDEX HINT 语法和效果 |
| Phase 3 | 代价模型测试 | 验证代价估算准确性 |
| Phase 3 | ANALYZE 测试 | 统计信息收集正确性 |
| 全部 | 回归测试 | 83 个测试文件持续通过 |

---

## 7. 预期性能提升

| 场景 | 优化前 | 优化后（预期） |
|------|--------|----------------|
| SF1 全表扫描 | 1.7s | <100ms（利用索引） |
| SF1 谓词过滤 | 过滤在 Executor | 过滤在 Storage |
| 60万行 WHERE id > 500000 | 返回全部 60万行 | 只返回满足条件的行 |

**目标**: 超越 SQLite 在分析型查询上的性能

---

## 8. 文件清单

### 新增文件
- `crates/storage/src/predicate.rs` - Predicate 类型定义
- `crates/optimizer/src/stats.rs` - 统计信息收集
- `executor/src/scan.rs` - 统一 ScanExecutor trait
- `sql/src/analyze.rs` - ANALYZE 命令

### 修改文件
- `crates/storage/src/engine.rs` - StorageEngine trait 新增接口
- `crates/storage/src/file_storage.rs` - FileStorage 实现
- `crates/storage/src/columnar/storage.rs` - ColumnarStorage 实现
- `executor/src/seq_scan.rs` - SeqScanExecutor 重构
- `executor/src/index_scan.rs` - IndexScanExecutor 实现
- `crates/optimizer/src/cost.rs` - 代价模型重构
- `crates/optimizer/src/path_selector.rs` - 路径选择器重构
- `crates/optimizer/src/rules.rs` - 优化规则新增
