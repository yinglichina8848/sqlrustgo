# 大数据量性能优化实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 全面提升 SQLRustGo 在大数据量场景（>10万行，1G-10G数据集）下的性能，使其达到或超过 SQLite 水平

**Architecture:** 三阶段实施 - Phase 1 谓词下推存储层，Phase 2 索引优化+统一ScanExecutor，Phase 3 新代价模型+统计信息收集

**Tech Stack:** Rust, StorageEngine trait, MemoryStorage/FileStorage/ColumnarStorage, B+Tree/HashIndex, Volcano Executor

---

## 阶段 1: 谓词下推存储层

### Task 1: 创建 Predicate 类型定义

**Files:**
- Create: `crates/storage/src/predicate.rs`
- Modify: `crates/storage/src/lib.rs` (导出新模块)

**Step 1: 创建 predicate.rs 文件**

```rust
// crates/storage/src/predicate.rs

use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Vec<Expr>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),
    IsNull(Box<Expr>),
    IsNotNull(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(String),
    Value(Value),
    Parameter(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexOp {
    Eq(i64),
    Range(i64, i64),
}
```

**Step 2: 更新 lib.rs 导出**

```rust
// crates/storage/src/lib.rs
pub mod predicate;
pub use predicate::{Predicate, Expr, IndexOp};
```

**Step 3: 提交**

```bash
git add crates/storage/src/predicate.rs crates/storage/src/lib.rs
git commit -m "feat(storage): add Predicate and IndexOp types"
```

---

### Task 2: StorageEngine trait 新增 scan_predicate 接口

**Files:**
- Modify: `crates/storage/src/engine.rs` (StorageEngine trait)

**Step 1: 添加 scan_predicate 方法签名**

```rust
// 在 StorageEngine trait 中添加:

/// 谓词下推扫描 - 存储层直接评估谓词，只返回匹配行
fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[usize]>,
) -> Result<ScanResult, StorageError>;

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub records: Vec<Record>,
    pub total: usize,
}
```

**Step 2: 提交**

```bash
git add crates/storage/src/engine.rs
git commit -m "feat(storage): add scan_predicate to StorageEngine trait"
```

---

### Task 3: MemoryStorage 实现 scan_predicate

**Files:**
- Modify: `crates/storage/src/engine.rs` (MemoryStorage impl)

**Step 1: 实现 scan_predicate**

```rust
// MemoryStorage 实现

fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[usize]>,
) -> Result<ScanResult, StorageError> {
    let table_data = self.tables.get(table)
        .ok_or(StorageError::TableNotFound(table.to_string()))?;
    
    // 1. 尝试使用索引
    if let Some((column, op)) = self.analyze_predicate_for_index(predicate) {
        if let Some(row_ids) = self.try_index_lookup(table, column, &op)? {
            let records = self.fetch_rows_by_ids(table_data, row_ids, projection);
            return Ok(ScanResult {
                total: records.len(),
                records,
            });
        }
    }

    // 2. 全表扫描 + 谓词过滤
    let filtered: Vec<Record> = table_data
        .iter()
        .filter(|row| self.eval_predicate(row, predicate))
        .cloned()
        .collect();

    let total = filtered.len();
    let records = self.apply_projection(filtered, projection);

    Ok(ScanResult { records, total })
}
```

**Step 2: 实现辅助方法**

```rust
// 添加到 MemoryStorage

fn analyze_predicate_for_index(&self, predicate: &Predicate) -> Option<(String, IndexOp)> {
    // 分析谓词，尝试匹配索引
    match predicate {
        Predicate::Eq(Expr::Column(col), Expr::Value(Value::Integer(i))) => {
            Some((col.clone(), IndexOp::Eq(*i)))
        }
        // ... 其他情况
        _ => None,
    }
}

fn try_index_lookup(&self, table: &str, column: &str, op: &IndexOp) -> Result<Option<Vec<u32>>, StorageError> {
    let index_key = format!("{}_{}", table, column);
    
    match op {
        IndexOp::Eq(val) => {
            if let Some(hash_idx) = self.hash_indexes.get(&index_key) {
                Ok(hash_idx.get(val))
            } else {
                Ok(None)
            }
        }
        IndexOp::Range(start, end) => {
            if let Some(btree_idx) = self.indexes.get(&index_key) {
                Ok(Some(btree_idx.range(*start, *end)))
            } else {
                Ok(None)
            }
        }
    }
}

fn eval_predicate(&self, row: &Record, predicate: &Predicate) -> bool {
    // 实现谓词评估逻辑
    match predicate {
        Predicate::Eq(l, r) => self.eval_expr(row, l) == self.eval_expr(row, r),
        Predicate::Lt(l, r) => self.eval_expr(row, l) < self.eval_expr(row, r),
        // ... 其他情况
        Predicate::And(l, r) => self.eval_predicate(row, l) && self.eval_predicate(row, r),
        Predicate::Or(l, r) => self.eval_predicate(row, l) || self.eval_predicate(row, r),
        Predicate::Not(p) => !self.eval_predicate(row, p),
        _ => true,
    }
}
```

**Step 3: 提交**

```bash
git add crates/storage/src/engine.rs
git commit -m "feat(storage): implement scan_predicate in MemoryStorage"
```

---

### Task 4: FileStorage 实现 scan_predicate

**Files:**
- Modify: `crates/storage/src/file_storage.rs`

**Step 1: 实现 FileStorage scan_predicate**

```rust
// FileStorage 实现
fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[usize]>,
) -> Result<ScanResult, StorageError> {
    // 类似 MemoryStorage，但读取磁盘上的 JSON 数据
    let table_data = self.load_table_data(table)?;
    let filtered = table_data.records
        .iter()
        .filter(|row| self.eval_predicate(row, predicate))
        .cloned()
        .collect();
    
    let total = filtered.len();
    let records = self.apply_projection(filtered, projection);
    
    Ok(ScanResult { records, total })
}
```

**Step 2: 提交**

```bash
git add crates/storage/src/file_storage.rs
git commit -m "feat(storage): implement scan_predicate in FileStorage"
```

---

### Task 5: ColumnarStorage 实现 scan_predicate

**Files:**
- Modify: `crates/storage/src/columnar/storage.rs`

**Step 1: 实现 ColumnarStorage scan_predicate**

```rust
// ColumnarStorage 实现
fn scan_predicate(
    &self,
    table: &str,
    predicate: &Predicate,
    projection: Option<&[usize]>,
) -> Result<ScanResult, StorageError> {
    let store = self.stores.get(table)
        .ok_or(StorageError::TableNotFound(table.to_string()))?;
    
    // 在列式存储上评估谓词
    let mask = self.eval_predicate_columnar(store, predicate)?;
    
    // 使用 mask 过滤行
    let records = store.filter_by_mask(&mask, projection)?;
    
    Ok(ScanResult {
        total: records.len(),
        records,
    })
}
```

**Step 2: 提交**

```bash
git add crates/storage/src/columnar/storage.rs
git commit -m "feat(storage): implement scan_predicate in ColumnarStorage"
```

---

### Task 6: SeqScanExecutor 接入 scan_predicate

**Files:**
- Modify: `crates/executor/src/seq_scan.rs`

**Step 1: 修改 SeqScanExecutor 使用 scan_predicate**

```rust
// SeqScanExecutor::init 修改
fn init(&mut self, ctx: &ExecutionContext) -> Result<(), ExecutorError> {
    let storage = ctx.storage();
    
    // 如果有谓词，使用 scan_predicate
    if let Some(predicate) = &self.predicate {
        let result = storage.scan_predicate(
            &self.table_name,
            predicate,
            self.projection.as_deref(),
        )?;
        self.records = result.records;
    } else {
        // 否则使用原有 scan
        self.records = storage.scan(&self.table_name)?;
    }
    
    Ok(())
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/seq_scan.rs
git commit -m "feat(executor): SeqScanExecutor uses scan_predicate"
```

---

### Task 7: 添加谓词过滤测试

**Files:**
- Create: `tests/storage/test_predicate_pushdown.rs`

**Step 1: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    
    #[test]
    fn test_scan_predicate_eq() {
        let storage = MemoryStorage::new();
        storage.insert("users", vec![1.into(), "Alice".into()]).unwrap();
        storage.insert("users", vec![2.into(), "Bob".into()]).unwrap();
        
        let predicate = Predicate::Eq(
            Box::new(Expr::Column("id".into())),
            Box::new(Expr::Value(1.into())),
        );
        
        let result = storage.scan_predicate("users", &predicate, None).unwrap();
        assert_eq!(result.records.len(), 1);
    }
    
    #[test]
    fn test_scan_predicate_range() {
        // 测试范围查询
    }
    
    #[test]
    fn test_scan_predicate_and() {
        // 测试 AND 谓词
    }
    
    #[test]
    fn test_scan_predicate_or() {
        // 测试 OR 谓词
    }
}
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-storage test_predicate
```

**Step 3: 提交**

```bash
git add tests/storage/test_predicate_pushdown.rs
git commit -m "test: add predicate pushdown tests"
```

---

### Task 8: 回归测试验证

**Files:**
- Run: 现有测试套件

**Step 1: 运行全部测试**

```bash
cargo test --workspace 2>&1 | tail -50
```

**Step 2: 运行 TPC-H 测试**

```bash
cargo test -p sqlrustgo-tpch 2>&1 | tail -30
```

**Step 3: 性能对比**

```bash
# 对比优化前后性能
./scripts/benchmark.sh --sf=1 --compare-sqlite
```

**Step 4: 提交**

```bash
git add -A
git commit -m "test: verify Phase 1 with regression tests"
```

---

## 阶段 2: 索引优化 + 统一 ScanExecutor

### Task 9: 创建统一 ScanExecutor trait

**Files:**
- Create: `crates/executor/src/scan.rs`

**Step 1: 定义 trait**

```rust
// crates/executor/src/scan.rs

pub trait ScanExecutor: Send {
    fn init(&mut self, ctx: &ExecutionContext) -> Result<(), ExecutorError>;
    fn next(&mut self) -> Result<Option<Record>, ExecutorError>;
    fn schema(&self) -> &Schema;
    fn stats(&self) -> &ScanStats;
    fn can_use_index(&self) -> bool { false }
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub rows_scanned: u64,
    pub estimated_cost: f64,
    pub used_index: bool,
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/scan.rs
git commit -m "feat(executor): add unified ScanExecutor trait"
```

---

### Task 10: 重构 SeqScanExecutor 实现 ScanExecutor

**Files:**
- Modify: `crates/executor/src/seq_scan.rs`

**Step 1: 实现 trait**

```rust
impl ScanExecutor for SeqScanExecutor {
    fn init(&mut self, ctx: &ExecutionContext) -> Result<(), ExecutorError> {
        // 现有逻辑
    }
    
    fn next(&mut self) -> Result<Option<Record>, ExecutorError> {
        // 现有逻辑
    }
    
    fn schema(&self) -> &Schema {
        &self.schema
    }
    
    fn stats(&self) -> &ScanStats {
        &self.stats
    }
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/seq_scan.rs
git commit -m "refactor(executor): SeqScanExecutor implements ScanExecutor trait"
```

---

### Task 11: 实现 IndexScanExecutor

**Files:**
- Modify: `crates/executor/src/index_scan.rs` (或新建)

**Step 1: 实现 IndexScanExecutor**

```rust
pub struct IndexScanExecutor {
    table_name: String,
    column: String,
    index_op: IndexOp,
    projection: Option<Vec<usize>>,
    records: Vec<Record>,
    pos: usize,
    stats: ScanStats,
}

impl ScanExecutor for IndexScanExecutor {
    fn init(&mut self, ctx: &ExecutionContext) -> Result<(), ExecutorError> {
        let storage = ctx.storage();
        
        // 使用索引扫描
        let result = storage.scan_with_index(
            &self.table_name,
            &self.column,
            &self.index_op,
            self.projection.as_deref(),
        )?;
        
        self.records = result.records;
        self.stats.used_index = true;
        Ok(())
    }
    
    fn can_use_index(&self) -> bool { true }
}
```

**Step 2: 提交**

```bash
git add crates/executor/src/index_scan.rs
git commit -m "feat(executor): implement IndexScanExecutor with ScanExecutor trait"
```

---

### Task 12: 优化器添加 IndexSelector 规则

**Files:**
- Modify: `crates/optimizer/src/rules.rs`

**Step 1: 添加规则**

```rust
pub struct IndexSelectorRule;

impl OptimizationRule for IndexSelectorRule {
    fn apply(&self, plan: &LogicalPlan) -> Result<LogicalPlan, OptimizerError> {
        // 分析谓词，检查是否有可用索引
        // 如果有，选择 IndexScan 而非 SeqScan
    }
}
```

**Step 2: 提交**

```bash
git add crates/optimizer/src/rules.rs
git commit -m "feat(optimizer): add IndexSelector optimization rule"
```

---

## 阶段 3: 新代价模型 + 统计信息收集

### Task 13: 创建统计信息模块

**Files:**
- Create: `crates/optimizer/src/stats.rs`

**Step 1: 定义统计类型**

```rust
// crates/optimizer/src/stats.rs

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
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

#[derive(Clone)]
pub struct IndexStats {
    pub index_name: String,
    pub cardinality: u64,
    pub selectiveness: f64,
}

pub struct StatsCache {
    tables: HashMap<String, TableStats>,
}
```

**Step 2: 提交**

```bash
git add crates/optimizer/src/stats.rs
git commit -m "feat(optimizer): add statistics collection module"
```

---

### Task 14: 重构代价模型

**Files:**
- Modify: `crates/optimizer/src/cost.rs`

**Step 1: 实现 CostModel**

```rust
pub struct CostModel {
    stats: StatsCache,
}

impl CostModel {
    pub fn estimate_seq_scan(&self, table: &TableStats) -> Cost {
        Cost {
            io_cost: table.row_count * ROW_SIZE / PAGE_SIZE,
            cpu_cost: table.row_count,
        }
    }
    
    pub fn estimate_index_scan(&self, table: &TableStats, index: &IndexStats, op: &IndexOp) -> Cost {
        let selectivity = self.compute_selectivity(index, op);
        let rows_to_read = (table.row_count as f64 * selectivity) as u64;
        Cost {
            io_cost: rows_to_read * ROW_SIZE / PAGE_SIZE,
            cpu_cost: rows_to_read + INDEX_LOOKUP_COST,
        }
    }
    
    pub fn select_best_path(&self, paths: Vec<ScanPath>) -> &ScanPath {
        paths.iter()
            .map(|p| (p, self.estimate_cost(p)))
            .min_by(|(_, c1), (_, c2)| c1.total().cmp(&c2.total()))
            .map(|(p, _)| p)
            .unwrap()
    }
}
```

**Step 2: 提交**

```bash
git add crates/optimizer/src/cost.rs
git commit -m "refactor(optimizer): rewrite cost model with statistics"
```

---

### Task 15: 实现 ANALYZE 命令

**Files:**
- Create: `crates/sql/src/analyze.rs`

**Step 1: 实现 ANALYZE**

```rust
pub struct AnalyzeStmt {
    pub table_name: String,
}

pub fn execute_analyze(storage: &dyn StorageEngine, stmt: AnalyzeStmt) -> Result<(), SqlError> {
    // 收集表统计信息
    let stats = collect_table_stats(storage, &stmt.table_name)?;
    
    // 存储到缓存
    StatsCache::get().update_table_stats(stmt.table_name, stats);
    
    Ok(())
}

fn collect_table_stats(storage: &dyn StorageEngine, table: &str) -> Result<TableStats, SqlError> {
    let records = storage.scan(table)?;
    // 计算 row_count, column_stats, index_stats
}
```

**Step 2: 提交**

```bash
git add crates/sql/src/analyze.rs
git commit -m "feat(sql): add ANALYZE command for statistics collection"
```

---

## 最终验证

### Task 16: 完整性能测试

**Step 1: SF0.1 测试**

```bash
./scripts/benchmark.sh --sf=0.1 --compare-sqlite
```

**Step 2: SF1 测试**

```bash
./scripts/benchmark.sh --sf=1 --compare-sqlite
```

**Step 3: 生成报告**

```bash
./scripts/generate-perf-report.sh
```

---

## 文件清单总结

### 新增文件
| 文件 | 用途 |
|------|------|
| `crates/storage/src/predicate.rs` | Predicate 类型定义 |
| `crates/executor/src/scan.rs` | 统一 ScanExecutor trait |
| `crates/optimizer/src/stats.rs` | 统计信息收集 |
| `crates/sql/src/analyze.rs` | ANALYZE 命令 |

### 修改文件
| 文件 | 改动 |
|------|------|
| `crates/storage/src/engine.rs` | StorageEngine trait 新增接口, MemoryStorage 实现 |
| `crates/storage/src/file_storage.rs` | FileStorage 实现 |
| `crates/storage/src/columnar/storage.rs` | ColumnarStorage 实现 |
| `crates/executor/src/seq_scan.rs` | SeqScanExecutor 重构 |
| `crates/executor/src/index_scan.rs` | IndexScanExecutor 实现 |
| `crates/optimizer/src/cost.rs` | 代价模型重构 |
| `crates/optimizer/src/rules.rs` | 优化规则新增 |

### 测试文件
| 文件 | 覆盖 |
|------|------|
| `tests/storage/test_predicate_pushdown.rs` | 谓词下推单元测试 |
| `tests/integration/test_tpch.rs` | TPC-H 集成测试 |
| `tests/benchmark/compare_sqlite.rs` | SQLite 性能对比 |
