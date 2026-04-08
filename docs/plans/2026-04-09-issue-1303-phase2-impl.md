# Issue #1303 Phase 2: 高级索引选择 - 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现基于成本的索引选择机制，支持多列索引、统计自动同步和 MySQL 风格 Hint

**Architecture:** 
- Task 1: OptimizerContext 统一注入 stats + cost_model 到各规则
- Task 3: CompositeKey Vec<Value> + lexicographic comparator 支持复合索引
- Task 2: StatsRegistry 回调机制解耦 storage 和 optimizer
- Task 4: IndexHint HashMap per-table + 优先级体系

**Tech Stack:** Rust, VolcanoExecutor pattern, StorageEngine trait, CBO cost model

---

## Task 1: OptimizerContext + CBO 集成

### 1.1 创建 OptimizerContext

**Files:**
- Create: `optimizer/src/context.rs`
- Modify: `optimizer/src/lib.rs` (导出)

**Step 1: 创建 context.rs 文件**

```rust
// optimizer/src/context.rs

use crate::stats::{StatisticsProvider, InMemoryStatisticsProvider};
use crate::cost::SimpleCostModel;
use std::sync::Arc;

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

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 2: 导出 OptimizerContext**

在 `optimizer/src/lib.rs` 添加:
```rust
pub mod context;
pub use context::OptimizerContext;
```

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 3: Commit**

```bash
git add optimizer/src/context.rs optimizer/src/lib.rs
git commit -m "feat(optimizer): add OptimizerContext for unified stats and cost injection"
```

---

### 1.2 重构 IndexSelect 使用 OptimizerContext

**Files:**
- Modify: `optimizer/src/rules.rs:856-940`

**Step 1: 修改 IndexSelect 结构体**

变更前:
```rust
pub struct IndexSelect {
    cost_model: SimpleCostModel,
    available_indexes: Vec<(String, String)>,
    selectivity_threshold: f64,
}
```

变更后:
```rust
pub struct IndexSelect {
    optimizer_ctx: Arc<OptimizerContext>,
    available_indexes: Vec<(String, String)>,
}
```

**Step 2: 修改 should_use_index 逻辑**

删除 selectivity_threshold 判断，改为纯成本比较:

```rust
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

    // 4. 纯成本决策
    if index_scan_cost < seq_scan_cost {
        IndexAccessMethod::IndexScan
    } else {
        IndexAccessMethod::SeqScan
    }
}
```

**Step 3: 修改 IndexSelect::new**

```rust
impl IndexSelect {
    pub fn new(optimizer_ctx: Arc<OptimizerContext>) -> Self {
        Self {
            optimizer_ctx,
            available_indexes: Vec::new(),
        }
    }
}
```

**Step 4: 更新所有调用 IndexSelect 的地方**

搜索: `IndexSelect::new()` 并添加 optimizer_ctx 参数

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 5: 更新测试**

修改 tests 中 IndexSelect 初始化:
```rust
let rule = IndexSelect::new(Arc::new(OptimizerContext::default()));
```

Run: `cargo test -p sqlrustgo-optimizer --lib index_select`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add optimizer/src/rules.rs
git commit -m "refactor(optimizer): IndexSelect uses OptimizerContext and pure CBO"
```

---

## Task 2: 多列索引

### 2.1 完善 CompositeKey

**Files:**
- Modify: `storage/src/bplus_tree/index.rs`

**Step 1: 添加 CompositeKey 结构体**

```rust
use std::cmp::Ordering;

/// 复合索引键
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

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
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

Run: `cargo check -p sqlrustgo-storage`
Expected: OK

**Step 2: Commit**

```bash
git add storage/src/bplus_tree/index.rs
git commit -m "feat(storage): add CompositeKey with lexicographic comparator"
```

---

### 2.2 创建 IndexRegistry

**Files:**
- Create: `storage/src/index_registry.rs`
- Modify: `storage/src/lib.rs` (导出)

**Step 1: 创建 index_registry.rs**

```rust
// storage/src/index_registry.rs

use crate::engine::IndexId;
use std::collections::HashMap;

/// 索引类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    BTree,
    Hash,
    CompositeBTree,
}

/// 索引元数据
#[derive(Debug, Clone)]
pub struct IndexMeta {
    pub id: IndexId,
    pub table_name: String,
    pub column_names: Vec<String>,
    pub index_type: IndexType,
}

/// 索引注册表
pub struct IndexRegistry {
    indexes: HashMap<IndexId, IndexMeta>,
    name_to_id: HashMap<String, IndexId>,
}

impl IndexRegistry {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn register(&mut self, meta: IndexMeta) {
        let id = meta.id;
        let name = format!("{}_{}", meta.table_name, meta.column_names.join("_"));
        self.name_to_id.insert(name, id);
        self.indexes.insert(id, meta);
    }

    pub fn get(&self, id: IndexId) -> Option<&IndexMeta> {
        self.indexes.get(&id)
    }

    pub fn get_by_name(&self, table: &str, columns: &[String]) -> Option<IndexId> {
        let name = format!("{}_{}", table, columns.join("_"));
        self.name_to_id.get(&name).copied()
    }

    pub fn next_id(&self) -> IndexId {
        IndexId(self.indexes.len() as u32)
    }
}

impl Default for IndexRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: 在 engine.rs 添加 IndexId**

```rust
// storage/src/engine.rs

/// 索引唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IndexId(pub u32);
```

**Step 3: 导出**

在 `storage/src/lib.rs`:
```rust
pub mod index_registry;
pub use index_registry::{IndexRegistry, IndexMeta, IndexType};
pub use engine::IndexId;
```

Run: `cargo check -p sqlrustgo-storage`
Expected: OK

**Step 4: Commit**

```bash
git add storage/src/index_registry.rs storage/src/engine.rs storage/src/lib.rs
git commit -m "feat(storage): add IndexRegistry and IndexId type"
```

---

### 2.3 扩展 StorageEngine Trait

**Files:**
- Modify: `storage/src/engine.rs`

**Step 1: 添加复合索引方法到 trait**

```rust
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

**Step 2: 在 MemoryStorage 实现**

在 `MemoryStorage` 结构体添加:
```rust
composite_indexes: HashMap<IndexId, CompositeBTreeIndex>,
```

实现方法:
```rust
fn create_composite_index(
    &mut self,
    table: &str,
    columns: Vec<String>,
) -> SqlResult<IndexId> {
    let id = self.index_registry.next_id();
    let mut index = CompositeBTreeIndex::new();
    
    // 填充现有数据
    let table_data = self.tables.get(table).ok_or("Table not found")?;
    let column_indices: Vec<usize> = columns
        .iter()
        .map(|col| {
            table_data
                .schema
                .columns
                .iter()
                .position(|c| c.name == *col)
                .ok_or(format!("Column {} not found", col))
        })
        .collect::<Result<Vec<_>, _>>()?;
    
    for (row_id, row) in table_data.records.iter().enumerate() {
        let key = CompositeKey::from_row(row, &column_indices);
        index.insert(key, row_id as u32);
    }
    
    let meta = IndexMeta {
        id,
        table_name: table.to_string(),
        column_names: columns,
        index_type: IndexType::CompositeBTree,
    };
    self.index_registry.register(meta);
    self.composite_indexes.insert(id, index);
    
    Ok(id)
}

fn search_composite_index(
    &self,
    index_id: IndexId,
    key: &CompositeKey,
) -> SqlResult<Vec<u32>> {
    self.composite_indexes
        .get(&index_id)
        .map(|idx| idx.search(key))
        .ok_or(format!("Composite index {:?} not found", index_id).into())
}

fn range_composite_index(
    &self,
    index_id: IndexId,
    start: &CompositeKey,
    end: &CompositeKey,
) -> SqlResult<Vec<u32>> {
    self.composite_indexes
        .get(&index_id)
        .map(|idx| idx.range_query(start, end))
        .ok_or(format!("Composite index {:?} not found", index_id).into())
}
```

Run: `cargo check -p sqlrustgo-storage`
Expected: OK (可能有类型错误需要调整)

**Step 3: Commit**

```bash
git add storage/src/engine.rs
git commit -m "feat(storage): extend StorageEngine with composite index APIs"
```

---

### 2.4 集成到 Executor

**Files:**
- Modify: `executor/src/local_executor.rs`

**Step 1: 添加复合索引执行**

在 `execute_index_scan` 中添加对 `Expr::BinaryExpr` 多列谓词的处理:

```rust
/// 执行复合索引扫描
fn execute_composite_index_scan(
    &self,
    plan: &IndexScanExec,
) -> SqlResult<ExecutorResult> {
    let index_id = plan.index_id(); // 需要在 IndexScanExec 添加
    let key = extract_composite_key(plan.predicate())?;
    
    let row_ids = self.storage.search_composite_index(index_id, &key)?;
    
    // Fetch rows...
}
```

**Step 2: Commit**

```bash
git add executor/src/local_executor.rs
git commit -m "feat(executor): support composite index execution"
```

---

## Task 3: StatsRegistry + ANALYZE 同步

### 3.1 创建 StatsRegistry

**Files:**
- Create: `optimizer/src/stats_registry.rs`
- Modify: `optimizer/src/lib.rs` (导出)

**Step 1: 创建 stats_registry.rs**

```rust
// optimizer/src/stats_registry.rs

use crate::stats::{StatisticsProvider, TableStats, StatsResult};
use std::sync::Arc;

/// 统计信息注册表 - 管理全局统计信息
#[derive(Clone)]
pub struct StatsRegistry {
    provider: Arc<dyn StatisticsProvider>,
}

impl StatsRegistry {
    pub fn new(provider: Arc<dyn StatisticsProvider>) -> Self {
        Self { provider }
    }

    /// 同步表的统计信息（由 storage ANALYZE 触发后调用）
    pub fn sync_table_stats(
        &self,
        table: &str,
        row_count: usize,
        column_stats: Vec<(String, ColumnStats)>,
    ) -> StatsResult<()> {
        let stats = TableStats::new(
            table.to_string(),
            row_count,
            column_stats,
        );
        self.provider.update_stats(table, stats)
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

**Step 2: 导出**

在 `optimizer/src/lib.rs`:
```rust
pub mod stats_registry;
pub use stats_registry::StatsRegistry;
```

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 3: Commit**

```bash
git add optimizer/src/stats_registry.rs optimizer/src/lib.rs
git commit -m "feat(optimizer): add StatsRegistry for stats synchronization"
```

---

### 3.2 ANALYZE 执行后触发同步

**Files:**
- Modify: `src/lib.rs` 或 `crates/executor/src/...`

**Step 1: 找到 ANALYZE 执行路径**

搜索 `Statement::Analyze` 在 `src/lib.rs` 中的处理

**Step 2: 在 ANALYZE 完成后调用 sync**

```rust
// 在 ANALYZE 执行后
if let Some(ref stats_registry) = global_stats_registry {
    stats_registry.sync_table_stats(
        &table_name,
        result.row_count,
        result.column_stats,
    )?;
}
```

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: integrate StatsRegistry with ANALYZE execution"
```

---

## Task 4: 索引 Hint

### 4.1 Parser 层添加 IndexHint

**Files:**
- Modify: `parser/src/parser.rs`
- Modify: `parser/src/token.rs`

**Step 1: 添加 Token**

在 `parser/src/token.rs` 添加:
```rust
pub enum Token {
    // ... existing ...
    USE,
    FORCE,
    IGNORE,
    INDEX,
}
```

**Step 2: 添加 AST 结构**

```rust
// parser/src/parser.rs

/// 索引 Hint 类型
#[derive(Debug, Clone, PartialEq)]
pub enum IndexHintType {
    UseIndex,
    ForceIndex,
    IgnoreIndex,
}

/// 索引 Hint
#[derive(Debug, Clone, PartialEq)]
pub struct IndexHint {
    pub table_alias: Option<String>,
    pub hint_type: IndexHintType,
    pub index_names: Vec<String>,
}

/// SelectStatement 扩展
pub struct SelectStatement {
    // ... existing fields ...
    pub index_hints: HashMap<String, IndexHint>,
}
```

**Step 3: 解析 Hint 语法**

```rust
fn parse_index_hint(&mut self) -> ParseResult<IndexHint> {
    // 解析: USE|FORCE|IGNORE INDEX (idx_name, ...)
    let hint_type = match self.parse_one()? {
        Token::USE => IndexHintType::UseIndex,
        Token::FORCE => IndexHintType::ForceIndex,
        Token::IGNORE => IndexHintType::IgnoreIndex,
        _ => return Err("Expected USE, FORCE, or IGNORE"),
    };
    
    self.expect(Token::INDEX)?;
    self.expect(Token::LPAREN)?;
    
    let mut index_names = Vec::new();
    loop {
        if let Token::RPAREN = self.peek()? {
            self.advance()?;
            break;
        }
        index_names.push(self.parse_identifier()?);
        if let Token::COMMA = self.peek()? {
            self.advance()?;
        }
    }
    
    Ok(IndexHint {
        table_alias: None,
        hint_type,
        index_names,
    })
}
```

Run: `cargo check -p sqlrustgo-parser`
Expected: OK

**Step 4: Commit**

```bash
git add parser/src/parser.rs parser/src/token.rs
git commit -m "feat(parser): add IndexHint AST and parsing"
```

---

### 4.2 Planner 传递 Hint 到 Optimizer

**Files:**
- Modify: `planner/src/planner.rs`

**Step 1: 修改 IndexSelect 添加 Hint 支持**

```rust
impl IndexSelect {
    /// 添加 Hint 强制使用的索引
    pub fn add_hint(&mut self, table_alias: &str, index_name: &str) {
        self.hints.insert(
            table_alias.to_string(),
            IndexHint {
                table_alias: Some(table_alias.to_string()),
                hint_type: IndexHintType::ForceIndex,
                index_names: vec![index_name.to_string()],
            },
        );
    }

    /// 获取 Hint 优先级: FORCE > IGNORE > USE > CBO
    fn resolve_index(&self, table: &str, default_index: Option<&str>) -> Option<String> {
        if let Some(hint) = self.hints.get(table) {
            match hint.hint_type {
                IndexHintType::ForceIndex => {
                    return hint.index_names.first().cloned();
                }
                IndexHintType::IgnoreIndex => {
                    // 返回第一个非 ignore 的索引
                    return default_index.filter(|idx| !hint.index_names.contains(&idx.to_string()));
                }
                IndexHintType::UseIndex => {
                    // USE 优先但可被 CBO 覆盖
                    return hint.index_names.first().cloned();
                }
            }
        }
        default_index.map(|s| s.to_string())
    }
}
```

**Step 2: Commit**

```bash
git add planner/src/planner.rs optimizer/src/rules.rs
git commit -m "feat(planner): propagate IndexHint to IndexSelect with priority"
```

---

## 最终验证

### 1. 运行所有测试

```bash
cargo test -p sqlrustgo-optimizer --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-executor --lib
```

### 2. 集成测试

```bash
cargo test --workspace --lib 2>&1 | tail -30
```

### 3. 推送

```bash
git push origin develop/v2.4.0
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
