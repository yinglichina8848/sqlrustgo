# Issue #1303: 自动选择索引 - 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现基于成本的索引选择机制，使查询计划器能够根据统计信息自动选择最优索引（HashIndex 或 B+Tree）

**Architecture:** 
- 扩展 IndexScanExec 物理计划节点，添加 predicate 和 access_method 字段
- 实现 IndexScanVolcanoExecutor 调用 StorageEngine 的 search_index/range_index API
- 增强 IndexSelect rule，添加谓词分类和成本估算逻辑
- 在 planner 中集成执行器创建

**Tech Stack:** Rust, VolcanoExecutor pattern, StorageEngine trait, CBO cost model

---

## 前提条件

- [ ] 了解 VolcanoExecutor trait (`executor/src/executor.rs:10-65`)
- [ ] 了解 StorageEngine search_index API (`storage/src/engine.rs:175`)
- [ ] 了解 IndexSelect rule (`optimizer/src/rules.rs:856-940`)
- [ ] 了解 IndexScanExec (`planner/src/physical_plan.rs:248-272`)

---

## Task 1: 添加 IndexAccessMethod 枚举和扩展 IndexScanExec

**Files:**
- Modify: `optimizer/src/cost.rs` (末尾添加)
- Modify: `planner/src/physical_plan.rs:248-272`

**Step 1: 添加 IndexAccessMethod 枚举到 cost.rs**

```rust
// optimizer/src/cost.rs 末尾添加

/// 索引访问方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexAccessMethod {
    /// 使用索引扫描 (Hash 或 B+Tree)
    IndexScan,
    /// 使用顺序扫描
    SeqScan,
}
```

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 2: 扩展 IndexScanExec 结构体**

```rust
// planner/src/physical_plan.rs 修改 IndexScanExec (约行 230-272)

#[derive(Debug, Clone)]
pub struct IndexScanExec {
    pub table_name: String,
    pub index_name: String,
    pub predicate: Option<Expr>,           // 新增
    pub access_method: IndexAccessMethod,  // 新增
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
```

Run: `cargo check -p sqlrustgo-planner`
Expected: OK

**Step 3: 更新 execute 方法返回错误**

```rust
impl PhysicalPlan for IndexScanExec {
    // ... existing methods ...

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Err("Use IndexScanVolcanoExecutor for execution".into())
    }
}
```

Run: `cargo check -p sqlrustgo-planner`
Expected: OK

**Step 4: Commit**

```bash
git add optimizer/src/cost.rs planner/src/physical_plan.rs
git commit -m "feat(planner): add IndexAccessMethod and extend IndexScanExec

- Add IndexAccessMethod enum to cost.rs
- Extend IndexScanExec with predicate, access_method, columns fields
- Mark execute() to delegate to IndexScanVolcanoExecutor"
```

---

## Task 2: 实现 IndexScanVolcanoExecutor

**Files:**
- Create: `executor/src/index_scan.rs`
- Modify: `executor/src/lib.rs` (导出新模块)
- Modify: `executor/src/local_executor.rs` (注册执行器)

**Step 1: 创建 index_scan.rs**

```rust
// executor/src/index_scan.rs

use crate::executor::{ExecutorResult, VolcanoExecutor};
use crate::Storage;
use sqlrustgo_storage::{Record, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::Arc;

/// Index scan executor using storage engine's index APIs
pub struct IndexScanVolcanoExecutor<S: StorageEngine> {
    storage: Arc<S>,
    table_name: String,
    column: String,
    predicate: Expr,
    rows: Vec<Record>,
    position: usize,
    schema: Schema,
}

impl<S: StorageEngine> IndexScanVolcanoExecutor<S> {
    pub fn new(
        storage: Arc<S>,
        table_name: String,
        column: String,
        predicate: Expr,
        schema: Schema,
    ) -> Self {
        Self {
            storage,
            table_name,
            column,
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
                let (col, val) = extract_column_value(left, right)?;
                self.column = col.to_string();
                self.storage.search_index(&self.table_name, &self.column, val)?
            }
            Expr::BinaryExpr { op: Operator::Gt, left, right } |
            Expr::BinaryExpr { op: Operator::Lt, left, right } |
            Expr::BinaryExpr { op: Operator::GtEq, left, right } |
            Expr::BinaryExpr { op: Operator::LtEq, left, right } => {
                let (col, start, end) = extract_range(&self.predicate)?;
                self.column = col.to_string();
                self.storage.range_index(&self.table_name, &self.column, start, end)?
            }
            _ => return Err(format!("Unsupported predicate: {:?}", self.predicate).into()),
        };

        for row_id in row_ids {
            if let Some(record) = self.storage.get_row(&self.table_name, row_id as usize)? {
                self.rows.push(record);
            }
        }
        Ok(())
    }

    fn next(&mut self) -> ExecutorResult<Option<Vec<Value>>> {
        if self.position < self.rows.len() {
            self.position += 1;
            Ok(Some(self.rows[self.position - 1].values.clone()))
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

fn extract_column_value(left: &Expr, right: &Expr) -> Result<(&str, i64), String> {
    let column = match (left, right) {
        (Expr::Column(col), Expr::Literal(Value::Integer(v))) => (col.as_str(), *v),
        (Expr::Literal(Value::Integer(v)), Expr::Column(col)) => (col.as_str(), *v),
        _ => return Err("Expected column = integer".into()),
    };
    Ok(column)
}

fn extract_range(predicate: &Expr) -> Result<(&str, i64, i64), String> {
    match predicate {
        Expr::BinaryExpr { op: Operator::Gt, left, right } => {
            let (col, val) = extract_column_value(left, right)?;
            Ok((col, val + 1, i64::MAX))
        }
        Expr::BinaryExpr { op: Operator::Lt, left, right } => {
            let (col, val) = extract_column_value(left, right)?;
            Ok((col, i64::MIN, val - 1))
        }
        Expr::BinaryExpr { op: Operator::GtEq, left, right } => {
            let (col, val) = extract_column_value(left, right)?;
            Ok((col, val, i64::MAX))
        }
        Expr::BinaryExpr { op: Operator::LtEq, left, right } => {
            let (col, val) = extract_column_value(left, right)?;
            Ok((col, i64::MIN, val))
        }
        _ => Err("Expected range predicate".into()),
    }
}
```

Run: `cargo check -p sqlrustgo-executor`
Expected: compilation errors (undefined types)

**Step 2: 修复导入**

添加必要的导入:
```rust
use sqlrustgo_planner::Expr;
use sqlrustgo_planner::Operator;
use sqlrustgo_storage::StorageEngine;
```

Run: `cargo check -p sqlrustgo-executor`
Expected: OK or fewer errors

**Step 3: 添加 lib.rs 导出**

```rust
// executor/src/lib.rs 添加
pub mod index_scan;
pub use index_scan::IndexScanVolcanoExecutor;
```

Run: `cargo check -p sqlrustgo-executor`
Expected: OK

**Step 4: Commit**

```bash
git add executor/src/index_scan.rs executor/src/lib.rs
git commit -m "feat(executor): add IndexScanVolcanoExecutor

- Implement IndexScanVolcanoExecutor using StorageEngine APIs
- Support equality (search_index) and range (range_index) queries
- Add helper functions for predicate extraction"
```

---

## Task 3: 集成到 LocalExecutor

**Files:**
- Modify: `executor/src/local_executor.rs`

**Step 1: 添加 execute_index_scan 函数**

找到 LocalExecutor impl 块，添加:

```rust
pub fn execute_index_scan(
    &self,
    plan: &IndexScanExec,
) -> ExecutorResult<Vec<Vec<Value>>> {
    let mut executor = IndexScanVolcanoExecutor::new(
        Arc::clone(&self.storage),
        plan.table_name.clone(),
        plan.index_name.clone(),
        plan.predicate.clone().unwrap(),  // IndexScan should have predicate
        plan.schema.clone(),
    );
    
    executor.init()?;
    
    let mut results = Vec::new();
    while let Some(row) = executor.next()? {
        results.push(row);
    }
    
    executor.close()?;
    Ok(results)
}
```

Run: `cargo check -p sqlrustgo-executor`
Expected: OK

**Step 2: 在 execute 方法中注册处理器**

在 `execute` 函数中添加:

```rust
match plan.name() {
    "SeqScan" => self.execute_seq_scan(...)?,
    "IndexScan" => self.execute_index_scan(...)?,  // 添加
    // ...
}
```

Run: `cargo check -p sqlrustgo-executor`
Expected: OK

**Step 3: Commit**

```bash
git add executor/src/local_executor.rs
git commit -m "feat(executor): integrate IndexScanVolcanoExecutor into LocalExecutor"
```

---

## Task 4: 增强 IndexSelect Rule

**Files:**
- Modify: `optimizer/src/rules.rs:856-940`

**Step 1: 添加 PredicateType 枚举和扩展 IndexSelect**

```rust
// 在 rules.rs 中 IndexSelect 前添加

enum PredicateType {
    Equality,
    Range,
    Unsupported,
}

// 修改 IndexSelect struct
pub struct IndexSelect {
    cost_model: SimpleCostModel,
    available_indexes: Vec<(String, String)>,
    selectivity_threshold: f64,  // 新增
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

    pub fn with_index(mut self, table: impl Into<String>, index: impl Into<String>) -> Self {
        self.available_indexes.push((table.into(), index.into()));
        self
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

    fn should_use_index(&self, table: &str, predicate: &Expr) -> IndexAccessMethod {
        // 检查是否有可用索引
        if !self.available_indexes.iter().any(|(t, _)| t == table) {
            return IndexAccessMethod::SeqScan;
        }

        // 检查谓词类型
        match self.classify_predicate(predicate) {
            PredicateType::Equality | PredicateType::Range => IndexAccessMethod::IndexScan,
            PredicateType::Unsupported => IndexAccessMethod::SeqScan,
        }
    }

    fn find_index_for_table(&self, table: &str) -> String {
        self.available_indexes
            .iter()
            .find(|(t, _)| t == table)
            .map(|(_, idx)| idx.clone())
            .unwrap_or_else(|| format!("{}_pkey", table))
    }
}
```

**Step 2: 修改 apply 方法**

更新 `impl Rule<Plan> for IndexSelect` 的 apply 方法:

```rust
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
                self.apply(input.as_mut())
            }
            _ => false,
        }
    }
}
```

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK (需要先添加 Plan::IndexScan 字段)

**Step 3: 添加 Plan::IndexScan 字段到 rules.rs**

在 rules.rs 的 Plan enum 中 (约行 20-24):

```rust
/// Index scan operation
IndexScan {
    table_name: String,
    index_name: String,
    predicate: Option<Expr>,
    access_method: IndexAccessMethod,  // 新增
},
```

Run: `cargo check -p sqlrustgo-optimizer`
Expected: OK

**Step 4: Commit**

```bash
git add optimizer/src/rules.rs
git commit -m "feat(optimizer): enhance IndexSelect rule with predicate classification"
```

---

## Task 5: 编写单元测试

**Files:**
- Modify: `optimizer/src/rules.rs` (tests section)
- Modify: `executor/src/index_scan.rs` (tests section)

**Step 1: 添加 IndexSelect 测试**

在 `optimizer/src/rules.rs` 的 tests 模块添加:

```rust
#[test]
fn test_index_select_equality_predicate() {
    let rule = IndexSelect::new()
        .with_index("users", "idx_id")
        .with_selectivity_threshold(0.1);

    let plan = Plan::Filter {
        predicate: Expr::BinaryExpr {
            left: Box::new(Expr::Column("id".to_string())),
            op: Operator::Eq,
            right: Box::new(Expr::Literal(Value::Integer(100))),
        },
        input: Box::new(Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        }),
    };

    let mut plan = plan;
    let applied = rule.apply(&mut plan);
    assert!(applied);
    
    match *plan {
        Plan::Filter { input, .. } => {
            match *input {
                Plan::IndexScan { table_name, index_name, .. } => {
                    assert_eq!(table_name, "users");
                    assert_eq!(index_name, "idx_id");
                }
                _ => panic!("Expected IndexScan"),
            }
        }
        _ => panic!("Expected Filter"),
    }
}

#[test]
fn test_index_select_range_predicate() {
    let rule = IndexSelect::new()
        .with_index("users", "idx_id");

    let plan = Plan::Filter {
        predicate: Expr::BinaryExpr {
            left: Box::new(Expr::Column("id".to_string())),
            op: Operator::Gt,
            right: Box::new(Expr::Literal(Value::Integer(100))),
        },
        input: Box::new(Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        }),
    };

    let mut plan = plan;
    let applied = rule.apply(&mut plan);
    assert!(applied);
    // 验证 IndexScan 被创建
}

#[test]
fn test_index_select_no_index_available() {
    let rule = IndexSelect::new();  // 没有添加任何索引

    let plan = Plan::Filter {
        predicate: Expr::BinaryExpr {
            left: Box::new(Expr::Column("id".to_string())),
            op: Operator::Eq,
            right: Box::new(Expr::Literal(Value::Integer(100))),
        },
        input: Box::new(Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        }),
    };

    let mut plan = plan;
    let applied = rule.apply(&mut plan);
    assert!(!applied);  // 不应该转换
}
```

**Step 2: 运行 optimizer 测试**

```bash
cargo test -p sqlrustgo-optimizer --lib index_select
```

Expected: 3 tests PASS

**Step 3: 添加 IndexScanVolcanoExecutor 测试**

```rust
// executor/src/index_scan.rs tests

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::MemoryStorage;
    use tempfile::TempDir;

    fn create_test_storage() -> MemoryStorage {
        let temp = TempDir::new().unwrap();
        MemoryStorage::new(temp.path().to_path_buf())
    }

    #[test]
    fn test_index_scan_equality() {
        let mut storage = create_test_storage();
        
        // 创建表并插入数据
        let schema = vec![
            ("id".to_string(), "INTEGER".to_string()),
            ("name".to_string(), "TEXT".to_string()),
        ];
        storage.create_table("users", &schema).unwrap();
        
        // 创建索引
        storage.create_hash_index("users", "id", 0).unwrap();
        
        // 插入测试数据
        let records = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ];
        storage.insert("users", records).unwrap();
        
        // 执行索引扫描
        let mut executor = IndexScanVolcanoExecutor::new(
            Arc::new(storage),
            "users".to_string(),
            "id".to_string(),
            Expr::BinaryExpr {
                left: Box::new(Expr::Column("id".to_string())),
                op: Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            Schema::new(vec![]),
        );
        
        executor.init().unwrap();
        let results: Vec<_> = executor.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0][0], Value::Integer(2));
        
        executor.close().unwrap();
    }
}
```

**Step 4: 运行 executor 测试**

```bash
cargo test -p sqlrustgo-executor --lib index_scan
```

Expected: tests PASS

**Step 5: Commit**

```bash
git add optimizer/src/rules.rs executor/src/index_scan.rs
git commit -m "test: add unit tests for IndexSelect and IndexScanVolcanoExecutor"
```

---

## Task 6: 集成测试和回归测试

**Step 1: 运行所有 optimizer 测试**

```bash
cargo test -p sqlrustgo-optimizer --lib
```

Expected: All tests PASS

**Step 2: 运行所有 executor 测试**

```bash
cargo test -p sqlrustgo-executor --lib
```

Expected: All tests PASS

**Step 3: 运行 planner 测试**

```bash
cargo test -p sqlrustgo-planner --lib
```

Expected: All tests PASS

**Step 4: 运行 workspace 测试**

```bash
cargo test --workspace --lib 2>&1 | tail -30
```

Expected: All tests PASS (或只有 pre-existing failures)

---

## Task 7: 最终提交

**Step 1: 推送所有 commits**

```bash
git push origin develop/v2.4.0
```

**Step 2: 验证 CI 状态**

```bash
gh pr status
```

---

## 验收清单

- [ ] IndexScanVolcanoExecutor 正确执行等值查询
- [ ] IndexScanVolcanoExecutor 正确执行范围查询
- [ ] IndexSelect rule 正确分类谓词类型
- [ ] IndexSelect rule 在有索引时正确转换计划
- [ ] 所有新增代码有单元测试
- [ ] 现有测试通过，无回归
- [ ] 代码已推送到 develop/v2.4.0

---

## 后续扩展 (Phase 2)

1. **成本模型集成**: 将 CboOptimizer.select_access_method 集成到 IndexSelect
2. **多列索引**: 支持复合索引 (a, b) = (1, 2)
3. **索引 hint**: 支持 SQL 注释强制使用特定索引
4. **统计信息更新**: ANALYZE TABLE 自动更新统计信息
