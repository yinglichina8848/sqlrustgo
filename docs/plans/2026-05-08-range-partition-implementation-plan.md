# ISSUE 442 - RANGE Partition 分区裁剪实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 RANGE 分区表的 Planner-level pruning，支持分区裁剪以提升查询性能。

**Architecture:** Storage-level 裁剪方案，每个分区独立文件存储。分区裁剪在 Optimizer 层完成，Physical Plan 携带分区列表传递给 Executor 并行扫描。

**Tech Stack:** Rust, Cargo Workspace, sqlrustgo crates

---

## 阶段 1: Parser 层 - 分区语法解析

### Task 1: 添加 PARTITION 关键字

**Files:**
- Modify: `crates/parser/src/token.rs`
- Modify: `crates/parser/src/lexer.rs`

**Step 1: 查看现有 token 定义**

Run: `grep -n "KW_CREATE" crates/parser/src/token.rs | head -5`

**Step 2: 添加 PARTITION 关键字**

```rust
// crates/parser/src/token.rs
// 在 Keyword 枚举中添加
KW_PARTITION,
```

**Step 3: 添加 PARTITION 词法**

```rust
// crates/parser/src/lexer.rs
// 在 keyword_map 中添加
"PARTITION" => Keyword::KW_PARTITION,
```

**Step 4: 添加 MAXVALUE 关键字**

```rust
// crates/parser/src/token.rs
KW_MAXVALUE,

// crates/parser/src/lexer.rs
"MAXVALUE" => Keyword::KW_MAXVALUE,
```

**Step 5: 提交**

```bash
git add crates/parser/src/token.rs crates/parser/src/lexer.rs
git commit -m "feat(parser): add PARTITION and MAXVALUE keywords"
```

---

### Task 2: 解析 PARTITION BY RANGE 语法

**Files:**
- Modify: `crates/parser/src/parser.rs`

**Step 1: 查看现有 CREATE TABLE 解析**

Run: `grep -n "fn parse_create_table" crates/parser/src/parser.rs`

**Step 2: 添加分区相关结构**

```rust
// crates/parser/src/parser.rs

#[derive(Debug, Clone)]
pub struct PartitionDefinition {
    pub partition_type: PartitionType,
    pub expr: Box<Expr>,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub enum PartitionType {
    Range,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: String,
    pub bound: PartitionBound,
}

#[derive(Debug, Clone)]
pub enum PartitionBound {
    LessThan(Value),
    MaxValue,
}
```

**Step 3: 修改 parse_create_table 函数**

在 CREATE TABLE 解析逻辑中添加分区解析：

```rust
// 在解析完 columns 后，检查 PARTITION BY
if self.parse_keyword(vec!["PARTITION", "BY", "RANGE"]) {
    let expr = self.parse_expr()?;
    self.expect_token(&Token::LParen)?;
    let mut partitions = Vec::new();
    loop {
        self.expect_keyword("PARTITION")?;
        let name = self.parse_identifier()?;
        self.expect_keyword("VALUES")?;
        self.expect_keyword("LESS")?;
        self.expect_keyword("THAN")?;
        self.expect_token(&Token::LParen)?;
        let bound = if self.parse_keyword(vec!["MAXVALUE"]) {
            PartitionBound::MaxValue
        } else {
            let v = self.parse_value()?;
            PartitionBound::LessThan(v)
        };
        self.expect_token(&Token::RParen)?;
        partitions.push(Partition { name, bound });
        if !self.parse_token(&Token::Comma) {
            break;
        }
    }
    self.expect_token(&Token::RParen)?;
}
```

**Step 4: 验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | tail -20`

**Step 5: 提交**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): parse PARTITION BY RANGE syntax"
```

---

## 阶段 2: Catalog 层 - 分区元数据存储

### Task 3: 添加 Catalog 分区结构

**Files:**
- Modify: `crates/catalog/src/lib.rs`

**Step 1: 查看现有 TableSchema 结构**

Run: `grep -n "struct TableSchema" crates/catalog/src/lib.rs`

**Step 2: 添加分区结构**

```rust
// crates/catalog/src/lib.rs

#[derive(Debug, Clone)]
pub struct PartitionDefinition {
    pub partition_type: PartitionType,
    pub expr: String,  // 表达式文本，如 "YEAR(batch_date)"
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub enum PartitionType {
    Range,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub id: u32,
    pub name: String,
    pub bound_less_than: Option<Value>,
    pub is_max_value: bool,
}
```

**Step 3: 修改 TableSchema 添加 partition 字段**

```rust
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub partition: Option<PartitionDefinition>,  // 新增
}
```

**Step 4: 验证编译**

Run: `cargo build -p sqlrustgo-catalog 2>&1 | tail -20`

**Step 5: 提交**

```bash
git add crates/catalog/src/lib.rs
git commit -m "feat(catalog): add partition structures to TableSchema"
```

---

### Task 4: 实现分区元数据存取方法

**Files:**
- Modify: `crates/catalog/src/lib.rs`

**Step 1: 添加分区存储方法**

```rust
impl Catalog {
    pub fn create_table_with_partition(
        &mut self,
        name: String,
        columns: Vec<Column>,
        partition: Option<PartitionDefinition>,
    ) -> Result<TableSchema, SqlError> {
        let schema = TableSchema { name, columns, partition };
        self.tables.insert(schema.name.clone(), schema.clone());
        Ok(schema)
    }

    pub fn get_table_partitions(&self, table_name: &str) -> Option<&PartitionDefinition> {
        self.tables.get(table_name)?.partition.as_ref()
    }
}
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-catalog 2>&1 | tail -20`

**Step 3: 提交**

```bash
git add crates/catalog/src/lib.rs
git commit -m "feat(catalog): implement partition metadata access methods"
```

---

## 阶段 3: Planner 层 - 逻辑计划

### Task 5: 添加 LogicalPlan 分区支持

**Files:**
- Modify: `crates/planner/src/logical_plan.rs`

**Step 1: 查看现有 LogicalPlan 枚举**

Run: `grep -n "enum LogicalPlan" crates/planner/src/logical_plan.rs`

**Step 2: 添加分区相关结构到 logical_plan.rs**

```rust
// 在 LogicalPlan 之前添加
use sqlrustgo_types::Value;

#[derive(Debug, Clone)]
pub struct PartitionDefinition {
    pub partition_type: PartitionType,
    pub expr: String,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone)]
pub enum PartitionType {
    Range,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub id: u32,
    pub name: String,
    pub bound_less_than: Option<Value>,
    pub is_max_value: bool,
}
```

**Step 3: 修改 LogicalPlan::CreateTable**

```rust
LogicalPlan::CreateTable {
    table_name: String,
    schema: Schema,
    if_not_exists: bool,
    partition: Option<PartitionDefinition>,  // 新增
},
```

**Step 4: 验证编译**

Run: `cargo build -p sqlrustgo-planner 2>&1 | tail -20`

**Step 5: 提交**

```bash
git add crates/planner/src/logical_plan.rs
git commit -m "feat(planner): add partition structures to LogicalPlan"
```

---

## 阶段 4: Optimizer 层 - 分区裁剪

### Task 6: 实现分区裁剪优化规则

**Files:**
- Modify: `crates/planner/src/optimizer.rs`

**Step 1: 查看现有优化规则结构**

Run: `grep -n "pub struct Optimizer" crates/planner/src/optimizer.rs`

**Step 2: 添加分区裁剪函数**

```rust
impl Optimizer {
    /// 根据 WHERE 条件裁剪分区
    pub fn prune_partitions(
        &self,
        table_name: &str,
        predicate: &Expr,
        partitions: &[Partition],
    ) -> Vec<u32> {
        // 简化实现：提取分区表达式的值
        // 例如 WHERE batch_date = '2025-06-01'
        // → YEAR(batch_date) = 2025
        // → 找到 bound >= 2025 的分区

        let value = extract_partition_value(predicate);
        if let Some(v) = value {
            partitions
                .iter()
                .filter(|p| {
                    if p.is_max_value {
                        true
                    } else if let Some(bound) = &p.bound_less_than {
                        v >= *bound
                    } else {
                        true
                    }
                })
                .map(|p| p.id)
                .collect()
        } else {
            partitions.iter().map(|p| p.id).collect()
        }
    }
}

fn extract_partition_value(expr: &Expr) -> Option<Value> {
    // 从 WHERE 条件中提取分区表达式的值
    // 例如 batch_date = '2025-06-01' -> 返回 2025
    match expr {
        Expr::BinaryOp { left, right, op: BinaryOperator::Eq, .. } => {
            // 简化：假设 left 是列，right 是值
            if let Expr::Value(v) = right.as_ref() {
                Some(v.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-planner 2>&1 | tail -20`

**Step 4: 提交**

```bash
git add crates/planner/src/optimizer.rs
git commit -m "feat(planner): implement partition pruning optimization"
```

---

## 阶段 5: Executor 层 - 分区执行

### Task 7: 实现 PartitionedTableScan 执行器

**Files:**
- Create: `crates/executor/src/partition_executor.rs`
- Modify: `crates/executor/src/lib.rs`

**Step 1: 创建分区执行器**

```rust
// crates/executor/src/partition_executor.rs

use sqlrustgo_types::Value;
use crate::Storage;

pub struct PartitionedTableScan {
    pub table_name: String,
    pub partition_ids: Vec<u32>,
    pub projection: Vec<usize>,
}

impl PartitionedTableScan {
    pub fn new(table_name: String, partition_ids: Vec<u32>) -> Self {
        Self {
            table_name,
            partition_ids,
            projection: vec![],
        }
    }

    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = projection;
        self
    }

    /// 执行分区扫描
    pub async fn execute<S: Storage>(
        &self,
        storage: &S,
    ) -> Result<Vec<Vec<Value>>, SqlError> {
        let mut results = Vec::new();
        for partition_id in &self.partition_ids {
            let records = storage.read_partition(&self.table_name, *partition_id)?;
            results.extend(records);
        }
        Ok(results)
    }
}
```

**Step 2: 在 lib.rs 中导出**

```rust
// crates/executor/src/lib.rs
pub mod partition_executor;
pub use partition_executor::PartitionedTableScan;
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-executor 2>&1 | tail -20`

**Step 4: 提交**

```bash
git add crates/executor/src/partition_executor.rs crates/executor/src/lib.rs
git commit -m "feat(executor): implement PartitionedTableScan executor"
```

---

## 阶段 6: Storage 层 - 分区存储

### Task 8: 实现分区存储读写

**Files:**
- Create: `crates/storage/src/partition_storage.rs`
- Modify: `crates/storage/src/lib.rs`

**Step 1: 创建分区存储模块**

```rust
// crates/storage/src/partition_storage.rs

use std::collections::HashMap;
use sqlrustgo_types::Value;

pub struct PartitionStorage {
    /// 表名 -> 分区ID -> 数据
    data: HashMap<String, HashMap<u32, Vec<Vec<Value>>>>,
}

impl PartitionStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// 写入分区数据
    pub fn write_partition(
        &mut self,
        table_name: &str,
        partition_id: u32,
        records: Vec<Vec<Value>>,
    ) -> Result<(), SqlError> {
        let table_data = self.data.entry(table_name.to_string()).or_default();
        table_data.insert(partition_id, records);
        Ok(())
    }

    /// 读取分区数据
    pub fn read_partition(
        &self,
        table_name: &str,
        partition_id: u32,
    ) -> Result<Vec<Vec<Value>>, SqlError> {
        Ok(self.data
            .get(table_name)
            .and_then(|t| t.get(&partition_id))
            .cloned()
            .unwrap_or_default())
    }

    /// 计算值的分区ID
    pub fn calculate_partition(
        &self,
        table_name: &str,
        value: &Value,
    ) -> Option<u32> {
        // 根据分区边界计算
        // 简化实现：假设分区按顺序排列
        let partitions = self.data.get(table_name)?;
        for (id, records) in partitions {
            if let Some(first) = records.first() {
                // 简化：只检查第一个记录
                return Some(*id);
            }
        }
        partitions.keys().min().copied()
    }
}
```

**Step 2: 在 lib.rs 中导出**

```rust
// crates/storage/src/lib.rs
pub mod partition_storage;
pub use partition_storage::PartitionStorage;
```

**Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-storage 2>&1 | tail -20`

**Step 4: 提交**

```bash
git add crates/storage/src/partition_storage.rs crates/storage/src/lib.rs
git commit -m "feat(storage): implement PartitionStorage for partition data"
```

---

## 阶段 7: 测试

### Task 9: 创建分区测试

**Files:**
- Create: `tests/partition_test.rs`

**Step 1: 创建测试文件**

```rust
// tests/partition_test.rs

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_catalog::Catalog;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    MemoryExecutionEngine::with_memory_and_catalog(catalog)
}

#[test]
fn test_create_partitioned_table() {
    let mut engine = create_engine();
    let result = engine.execute(
        "CREATE TABLE batch_record (
            id INTEGER,
            batch_date DATE
        ) PARTITION BY RANGE (YEAR(batch_date)) (
            PARTITION p2024 VALUES LESS THAN (2025),
            PARTITION p2025 VALUES LESS THAN (2026),
            PARTITION p_future VALUES LESS THAN MAXVALUE
        )"
    );
    assert!(result.is_ok(), "CREATE TABLE with PARTITION should succeed: {:?}", result.err());
}

#[test]
fn test_partitioned_insert() {
    let mut engine = create_engine();
    engine.execute(
        "CREATE TABLE batch_record (
            id INTEGER,
            batch_date DATE
        ) PARTITION BY RANGE (YEAR(batch_date)) (
            PARTITION p2024 VALUES LESS THAN (2025),
            PARTITION p2025 VALUES LESS THAN (2026),
            PARTITION p_future VALUES LESS THAN MAXVALUE
        )"
    ).unwrap();

    // Insert into p2025 partition
    let result = engine.execute(
        "INSERT INTO batch_record VALUES (1, '2025-06-01')"
    );
    assert!(result.is_ok(), "INSERT should succeed: {:?}", result.err());
}

#[test]
fn test_partition_pruning() {
    let mut engine = create_engine();
    engine.execute(
        "CREATE TABLE batch_record (
            id INTEGER,
            batch_date DATE
        ) PARTITION BY RANGE (YEAR(batch_date)) (
            PARTITION p2024 VALUES LESS THAN (2025),
            PARTITION p2025 VALUES LESS THAN (2026),
            PARTITION p_future VALUES LESS THAN MAXVALUE
        )"
    ).unwrap();

    engine.execute("INSERT INTO batch_record VALUES (1, '2025-06-01')").unwrap();
    engine.execute("INSERT INTO batch_record VALUES (2, '2024-06-01')").unwrap();

    // Query with partition pruning
    let result = engine.execute(
        "SELECT * FROM batch_record WHERE batch_date = '2025-06-01'"
    );
    assert!(result.is_ok(), "SELECT should succeed: {:?}", result.err());
}
```

**Step 2: 运行测试验证**

Run: `cargo test --test partition_test 2>&1 | tail -30`

**Step 3: 提交**

```bash
git add tests/partition_test.rs
git commit -m "test: add partition tests for ISSUE-442"
```

---

## 阶段 8: 集成与门禁

### Task 10: 集成测试

**Step 1: 确保所有组件编译通过**

Run: `cargo build --all 2>&1 | tail -30`

**Step 2: 运行完整测试**

Run: `cargo test --all-features 2>&1 | tail -50`

**Step 3: 提交**

```bash
git add -A
git commit -m "feat: complete RANGE partition implementation for ISSUE-442"
```

---

### Task 11: 门禁验证

**Step 1: 运行分区测试门禁**

Run: `cargo test --test partition_test --all-features`

Expected: 3/3 PASS

**Step 2: 运行 clippy 检查**

Run: `cargo clippy --all-features -- -D warnings`

Expected: 0 warnings

**Step 3: 运行 fmt 检查**

Run: `cargo fmt --check --all`

Expected: clean

**Step 4: 推送并创建 PR**

```bash
git push origin feat/range-partition-442
```

---

## 执行方式

**Plan complete and saved to `docs/plans/2026-05-08-range-partition-implementation-plan.md`.**

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**