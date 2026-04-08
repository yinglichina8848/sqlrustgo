# Issue #1303 设计文档：查询计划器 - 自动选择索引

## 1. 目标与范围

### 1.1 目标
实现基于成本的索引选择机制，使查询计划器能够根据统计信息自动选择最优索引（HashIndex 或 B+Tree），提升查询性能。

### 1.2 范围
**Phase 1（当前实现）**：
- 支持等值查询 `=` → HashIndex (O(1) 查找)
- 支持范围查询 `>`、`<`、`BETWEEN` → B+Tree range_index
- 集成成本模型到物理计划生成
- 完善 IndexScanExec 物理计划节点
- 实现 IndexScanVolcanoExecutor 执行器

**Phase 2（后续扩展）**：
- 多列索引支持
- JOIN 条件索引选择
- LIKE 查询索引支持
- 索引 hint 机制

---

## 2. 现状分析

### 2.1 已有基础设施

| 组件 | 文件 | 状态 |
|------|------|------|
| 统计信息 | `optimizer/src/stats.rs` | `TableStats`, `ColumnStats`, `eq_selectivity()` |
| 成本模型 | `optimizer/src/cost.rs` | `SimpleCostModel`, `select_access_method()` |
| 存储索引API | `storage/src/engine.rs` | `search_index()`, `range_index()`, `create_hash_index()` |
| 逻辑计划Rule | `optimizer/src/rules.rs` | `IndexSelect` rule (基础版本) |
| 物理计划 | `planner/src/physical_plan.rs` | `IndexScanExec` (存根，空实现) |

### 2.2 核心问题

```
IndexSelect rule (逻辑层)
    ↓ 转换 Filter+TableScan → IndexScan
IndexScanExec (物理层)
    ↓ 返回 vec![] 空结果
IndexScanVolcanoExecutor (执行层)
    ↓ 不存在
存储层 search_index/range_index
    ↓ 未被调用
实际数据
```

### 2.3 数据流

```
SQL: SELECT * FROM users WHERE id = 100
           ↓
逻辑计划: Filter(predicate: id = 100) → TableScan(users)
           ↓ IndexSelect rule (判断是否用索引)
物理计划: Filter → IndexScan(users, idx_id, id = 100)
           ↓ planner 创建执行器
执行器: IndexScanVolcanoExecutor
           ↓ 调用 storage.search_index("users", "id", 100)
结果: Vec<u32> (匹配的 row IDs)
```

---

## 3. 架构设计

### 3.1 组件关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Query Planning                             │
├─────────────────────────────────────────────────────────────────┤
│  Logical Plan    │  IndexSelect Rule  │  Cost Model            │
│  Filter → Table  │  ─────────────────▶│───select_access_method│
│                  │  should_use_index() │  selectivity           │
└──────────────────┴─────────────────────┴───────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Physical Plan                              │
├─────────────────────────────────────────────────────────────────┤
│  IndexScanExec {                                               │
│    table_name: String,                                         │
│    index_name: String,                                         │
│    predicate: Option<Expr>,                                    │
│    access_method: IndexAccessMethod,  // NEW                   │
│  }                                                             │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Executor Layer                             │
├─────────────────────────────────────────────────────────────────┤
│  IndexScanVolcanoExecutor {  // NEW                              │
│    storage: Arc<dyn StorageEngine>,                             │
│    table_name: String,                                         │
│    index_name: String,                                         │
│    predicate: Expr,                                            │
│    rows: Vec<Record>,        // fetched from storage            │
│    position: usize,                                           │
│  }                                                             │
│                                                                 │
│  impl VolcanoExecutor for IndexScanVolcanoExecutor {           │
│    fn init() {                                                 │
│      let row_ids = storage.search_index(table, column, key);   │
│      for id in row_ids {                                       │
│        rows.push(storage.get_row(id)?);                          │
│      }                                                         │
│    }                                                           │
│                                                                 │
│    fn next() → Option<Record> {                                │
│      if position < rows.len() {                                │
│        position += 1;                                           │
│        Some(rows[position-1].clone())                          │
│      } else { None }                                            │
│    }                                                           │
│  }                                                             │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Storage Layer                              │
├─────────────────────────────────────────────────────────────────┤
│  MemoryStorage / FileStorage                                    │
│                                                                 │
│  search_index(table, column, key) → Vec<u32>  // HashIndex    │
│  range_index(table, column, start, end) → Vec<u32>  // B+Tree │
│  get_row(table, row_id) → Option<Record>                      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 成本模型集成

```rust
// cost.rs - CboOptimizer::select_access_method
pub fn select_access_method(
    &self,
    table: &str,
    column: &str,
    selectivity_threshold: f64,  // 默认 0.1
) -> IndexAccessMethod {
    let seq_scan_cost = self.estimate_scan_cost(table);
    let index_scan_cost = self.estimate_index_scan_cost(table, column);

    let selectivity = self.stats_provider
        .map(|p| p.selectivity(table, column))
        .unwrap_or(0.1);

    if selectivity < selectivity_threshold && index_scan_cost < seq_scan_cost {
        IndexAccessMethod::IndexScan
    } else {
        IndexAccessMethod::SeqScan
    }
}
```

---

## 4. 详细设计

### 4.1 IndexAccessMethod 枚举

**文件**: `optimizer/src/cost.rs` (新增)

```rust
/// 索引访问方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexAccessMethod {
    /// 使用索引扫描 (Hash 或 B+Tree)
    IndexScan,
    /// 使用顺序扫描
    SeqScan,
}
```

### 4.2 IndexScanExec 扩展

**文件**: `planner/src/physical_plan.rs`

**现状** (行 248-272):
```rust
pub struct IndexScanExec {
    pub table_name: String,
    pub index_name: String,
    pub schema: Schema,
}

impl PhysicalPlan for IndexScanExec {
    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])  // 空实现 ❌
    }
}
```

**修改后**:
```rust
pub struct IndexScanExec {
    pub table_name: String,
    pub index_name: String,
    pub predicate: Option<Expr>,           // 新增
    pub access_method: IndexAccessMethod,  // 新增: Hash 或 Range
    pub schema: Schema,
    pub columns: Vec<String>,              // 新增: 要读取的列
}

impl IndexScanExec {
    pub fn new(
        table_name: String,
        index_name: String,
        predicate: Option<Expr>,
        access_method: IndexAccessMethod,
        schema: Schema,
        columns: Vec<String>,
    ) -> Self {
        Self {
            table_name,
            index_name,
            predicate,
            access_method,
            schema,
            columns,
        }
    }
}

impl PhysicalPlan for IndexScanExec {
    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "IndexScan" }
    fn table_name(&self) -> &str { &self.table_name }
    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        // 执行逻辑委托给 IndexScanVolcanoExecutor
        Err("Use IndexScanVolcanoExecutor for execution".into())
    }
    fn as_any(&self) -> &dyn Any { self }
}
```

### 4.3 IndexScanVolcanoExecutor 实现

**文件**: `executor/src/index_scan.rs` (新增)

```rust
use crate::executor::{ExecutorResult, VolcanoExecutor};
use crate::Storage;
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::Value;
use std::sync::Arc;

/// Index scan executor using storage engine's index APIs
pub struct IndexScanVolcanoExecutor<S: StorageEngine> {
    storage: Arc<S>,
    table_name: String,
    index_name: String,
    predicate: Expr,
    rows: Vec<Vec<Value>>,
    position: usize,
    schema: Schema,
}

impl<S: StorageEngine> IndexScanVolcanoExecutor<S> {
    pub fn new(
        storage: Arc<S>,
        table_name: String,
        index_name: String,
        predicate: Expr,
        schema: Schema,
    ) -> Self {
        Self {
            storage,
            table_name,
            index_name,
            predicate,
            rows: Vec::new(),
            position: 0,
            schema,
        }
    }
}

impl<S: StorageEngine> VolcanoExecutor for IndexScanVolcanoExecutor<S> {
    fn init(&mut self) -> ExecutorResult<()> {
        let row_ids = match &self.predicate {
            Expr::BinaryExpr { op: Operator::Eq, left, right } => {
                // 提取列名和值
                let (column, value) = extract_column_value(left, right)?;
                // 调用 HashIndex search
                self.storage.search_index(&self.table_name, column, value)?
            }
            Expr::BinaryExpr { op: Operator::Gt, left, right } |
            Expr::BinaryExpr { op: Operator::Lt, left, right } |
            Expr::BinaryExpr { op: Operator::GtEq, left, right } |
            Expr::BinaryExpr { op: Operator::LtEq, left, right } => {
                // 范围查询
                let (column, start, end) = extract_range(&self.predicate)?;
                self.storage.range_index(&self.table_name, column, start, end)?
            }
            _ => return Err(format!("Unsupported predicate: {:?}", self.predicate).into()),
        };

        // 根据 row_ids 获取完整行数据
        for row_id in row_ids {
            if let Some(record) = self.storage.get_row(&self.table_name, row_id as usize)? {
                self.rows.push(record.into_iter().map(|v| v).collect());
            }
        }
        Ok(())
    }

    fn next(&mut self) -> ExecutorResult<Option<Vec<Value>>> {
        if self.position < self.rows.len() {
            self.position += 1;
            Ok(Some(self.rows[self.position - 1].clone()))
        } else {
            Ok(None)
        }
    }

    fn close(&mut self) -> ExecutorResult<()> {
        self.rows.clear();
        self.position = 0;
        Ok(())
    }

    fn schema(&self) -> &Schema { &self.schema }
    fn name(&self) -> &str { "IndexScan" }
    fn is_initialized(&self) -> bool { !self.rows.is_empty() || self.position > 0 }
}

// ============ 辅助函数 ============

fn extract_column_value(expr: &Expr, right: &Expr) -> Result<(&str, i64), String> {
    // 从 BinaryExpr 提取列名和比较值
    // 例如: id = 100 → ("id", 100)
    todo!("Implement column/value extraction from predicate")
}

fn extract_range(predicate: &Expr) -> Result<(&str, i64, i64), String> {
    // 从范围谓词提取列名和范围
    // 例如: id > 100 AND id < 200 → ("id", 100, 200)
    todo!("Implement range extraction from predicate")
}
```

### 4.4 IndexSelect Rule 增强

**文件**: `optimizer/src/rules.rs`

**修改 `IndexSelect` rule**:

```rust
pub struct IndexSelect {
    cost_model: SimpleCostModel,
    available_indexes: Vec<(String, String)>,  // (table, index_name)
    selectivity_threshold: f64,  // 新增: 默认 0.1
}

impl IndexSelect {
    pub fn new() -> Self {
        Self {
            cost_model: SimpleCostModel::default_model(),
            available_indexes: Vec::new(),
            selectivity_threshold: 0.1,
        }
    }

    pub fn with_selectivity_threshold(mut self, threshold: f64) -> Self {
        self.selectivity_threshold = threshold;
        self
    }

    fn should_use_index(&self, table: &str, predicate: &Expr) -> IndexAccessMethod {
        // 1. 检查是否有可用索引
        if !self.available_indexes.iter().any(|(t, _)| t == table) {
            return IndexAccessMethod::SeqScan;
        }

        // 2. 检查谓词类型
        let predicate_type = self.classify_predicate(predicate);
        if predicate_type == PredicateType::Unsupported {
            return IndexAccessMethod::SeqScan;
        }

        // 3. 成本估算 (简化版本，后续集成 CboOptimizer)
        let use_index = match predicate_type {
            PredicateType::Equality => true,  // HashIndex 总是更快
            PredicateType::Range => {
                // 后续添加成本估算
                true
            }
        };

        if use_index {
            IndexAccessMethod::IndexScan
        } else {
            IndexAccessMethod::SeqScan
        }
    }

    fn classify_predicate(&self, expr: &Expr) -> PredicateType {
        match expr {
            Expr::BinaryExpr { op: Operator::Eq, .. } => PredicateType::Equality,
            Expr::BinaryExpr { op: Operator::Gt, .. } |
            Expr::BinaryExpr { op: Operator::Lt, .. } |
            Expr::BinaryExpr { op: Operator::GtEq, .. } |
            Expr::BinaryExpr { op: Operator::LtEq, .. } => PredicateType::Range,
            _ => PredicateType::Unsupported,
        }
    }
}

enum PredicateType {
    Equality,    // =
    Range,       // >, <, >=, <=, BETWEEN
    Unsupported,
}

impl Rule<Plan> for IndexSelect {
    fn name(&self) -> &str { "IndexSelect" }

    #[allow(clippy::only_used_in_recursion)]
    fn apply(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Filter { input, predicate } => {
                if let Plan::TableScan { table_name, projection } = input.as_ref() {
                    let method = self.should_use_index(table_name, predicate);
                    if method == IndexAccessMethod::IndexScan {
                        let index_name = self.find_index_for_table(table_name);
                        let new_plan = Plan::IndexScan {
                            table_name: table_name.clone(),
                            index_name,
                            predicate: Some(predicate.clone()),
                            access_method: method,
                        };
                        **input = new_plan;
                        return true;
                    }
                }
                // 递归检查嵌套
                self.apply(input.as_mut())
            }
            _ => false,
        }
    }
}
```

### 4.5 Planner 集成

**文件**: `planner/src/planner.rs`

修改 `create_physical_plan_internal` 添加 IndexScan 处理:

```rust
fn create_physical_plan_internal(&mut self, plan: LogicalPlan) -> PlannerResult<Box<dyn PhysicalPlan>> {
    match plan {
        // ... existing cases ...

        LogicalPlan::IndexScan { table_name, index_name, predicate } => {
            // 从 catalog 获取表信息
            let table_info = self.catalog.get_table(&table_name)?;
            let schema = Schema::from_table_info(&table_info);

            // 确定访问方法
            let access_method = self.determine_access_method(&predicate);

            Ok(Box::new(IndexScanExec::new(
                table_name,
                index_name,
                Some(predicate),
                access_method,
                schema,
                table_info.columns.iter().map(|c| c.name.clone()).collect(),
            )))
        }

        // ... existing cases ...
    }
}
```

---

## 5. 实现步骤

### Phase 1.1: 完善 IndexScanExec (半天)
- [ ] 扩展 `IndexScanExec` 结构体，添加必要字段
- [ ] 添加 `IndexAccessMethod` 枚举
- [ ] 更新 `execute()` 返回错误（执行委托给执行器）

### Phase 1.2: 实现 IndexScanVolcanoExecutor (1天)
- [ ] 创建 `executor/src/index_scan.rs`
- [ ] 实现 `VolcanoExecutor` trait
- [ ] 实现 `search_index` 调用（等值查询）
- [ ] 实现 `range_index` 调用（范围查询）
- [ ] 添加辅助函数提取谓词中的列和值

### Phase 1.3: 集成到 LocalExecutor (半天)
- [ ] 在 `local_executor.rs` 添加 `execute_index_scan`
- [ ] 在 `execute_plan` 中注册 IndexScan 处理器
- [ ] 传递 StorageEngine 给 IndexScanVolcanoExecutor

### Phase 1.4: 增强 IndexSelect Rule (半天)
- [ ] 添加 `PredicateType` 分类
- [ ] 实现 `should_use_index` 逻辑
- [ ] 添加 `selectivity_threshold` 配置
- [ ] 在 `apply` 中正确转换到 IndexScan

### Phase 1.5: 成本模型集成 (半天)
- [ ] 将 `CboOptimizer.select_access_method` 集成到 `IndexSelect`
- [ ] 添加统计信息查询
- [ ] 添加 selectivity_threshold 配置项

### Phase 1.6: 测试验证 (1天)
- [ ] 添加单元测试: `test_index_select_equality`
- [ ] 添加单元测试: `test_index_select_range`
- [ ] 添加集成测试: `test_index_scan_executor`
- [ ] 添加成本模型测试: `test_cost_based_selection`
- [ ] 运行现有回归测试

---

## 6. 测试计划

### 6.1 单元测试

```rust
// optimizer/src/rules.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_index_select_equality() {
        // Given: Filter(id = 100) → TableScan("users")
        // When: IndexSelect rule applied
        // Then: Filter → IndexScan("users", "idx_id", id = 100)
    }

    #[test]
    fn test_index_select_range() {
        // Given: Filter(id > 100 AND id < 200) → TableScan("users")
        // When: IndexSelect rule applied
        // Then: Filter → IndexScan with range predicate
    }

    #[test]
    fn test_index_select_no_index_available() {
        // Given: Filter(id = 100) → TableScan("unknown_table")
        // When: No index available
        // Then: No transformation (remains TableScan)
    }

    #[test]
    fn test_selectivity_threshold() {
        // Given: High selectivity (many rows match)
        // When: selectivity > threshold
        // Then: Prefer SeqScan over IndexScan
    }
}
```

### 6.2 集成测试

```rust
// executor/src/index_scan.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_index_scan_equality() {
        // Setup storage with index on "id" column
        // Execute: SELECT * FROM users WHERE id = 100
        // Verify: Returns correct row
    }

    #[test]
    fn test_index_scan_range() {
        // Setup storage with B+Tree index on "id" column
        // Execute: SELECT * FROM users WHERE id > 100 AND id < 200
        // Verify: Returns rows with id in range [100, 200]
    }

    #[test]
    fn test_index_scan_empty_result() {
        // Execute: SELECT * FROM users WHERE id = 99999 (not exist)
        // Verify: Returns empty result
    }
}
```

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 谓词解析复杂 | 实现延期 | 先支持简单 BinaryExpr，逐步扩展 |
| 成本模型不准确 | 选择次优索引 | 添加 selectivity_threshold 配置，允许调优 |
| 统计信息过时 | 选择错误 | 定期 ANALYZE，或添加 stale 检测 |

---

## 8. 验收标准

- [ ] `IndexScanVolcanoExecutor` 正确执行等值查询
- [ ] `IndexScanVolcanoExecutor` 正确执行范围查询
- [ ] `IndexSelect` rule 正确选择 HashIndex vs B+Tree
- [ ] 成本模型集成，选择性与阈值正确工作
- [ ] 所有新增代码有单元测试
- [ ] 现有测试通过，无回归
- [ ] 集成测试验证端到端流程

---

## 9. 参考文件

| 文件 | 用途 |
|------|------|
| `optimizer/src/stats.rs` | TableStats, ColumnStats, eq_selectivity() |
| `optimizer/src/cost.rs` | SimpleCostModel, select_access_method() |
| `optimizer/src/rules.rs` | IndexSelect rule (行 856-940) |
| `storage/src/engine.rs` | StorageEngine trait, search_index(), range_index() |
| `planner/src/physical_plan.rs` | IndexScanExec (行 248-272) |
| `planner/src/planner.rs` | create_physical_plan_internal |
| `executor/src/executor.rs` | VolcanoExecutor trait, SeqScanVolcanoExecutor |
| `executor/src/filter.rs` | FilterVolcanoExecutor 参考 |

---

## 10. 附录：相关类型定义

### 10.1 StorageEngine trait 相关方法

```rust
// storage/src/engine.rs
pub trait StorageEngine: Send + Sync {
    fn search_index(&self, table: &str, column: &str, key: i64) -> Vec<u32>;
    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32>;
    fn get_row(&self, table: &str, row_index: usize) -> SqlResult<Option<Record>>;
    // ...
}
```

### 10.2 VolcanoExecutor trait

```rust
// executor/src/executor.rs
pub trait VolcanoExecutor {
    fn init(&mut self) -> ExecutorResult<()>;
    fn next(&mut self) -> ExecutorResult<Option<Vec<Value>>>;
    fn close(&mut self) -> ExecutorResult<()>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn is_initialized(&self) -> bool;
}
```
