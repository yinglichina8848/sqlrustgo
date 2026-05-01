# FULL OUTER JOIN Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 FULL OUTER JOIN 支持，使用 HashJoin 算法和 3 Phase 去重策略

**Architecture:** 基于现有的 HashJoin 实现，在 `execute_hash_join()` 中添加 `JoinType::Full` 分支。Phase 1 执行标准 LEFT JOIN，Phase 2 收集已匹配右表键，Phase 3 输出右表未匹配行。三个输出集合互斥，无需显式去重。

**Tech Stack:** Rust, sqlrustgo-parser, sqlrustgo-executor

---

## 1. 文件结构

| 文件 | 修改类型 | 职责 |
|------|----------|------|
| `crates/parser/src/parser.rs` | 修改 | 添加 `Token::Full` → `JoinType::Full` 解析 |
| `crates/executor/src/local_executor.rs` | 修改 | 实现 `JoinType::Full` 分支 |
| `crates/executor/src/parallel_executor.rs` | 修改 | 并行执行器支持 `JoinType::Full` |
| `crates/executor/tests/join_test.rs` | 新增 | FULL OUTER JOIN 单元测试 |

---

## 2. Task 1: Parser 层 - FULL 关键字解析

**Files:**
- Modify: `crates/parser/src/parser.rs:1135-1175` (parse_join_clause 函数)

- [ ] **Step 1: 找到 parse_join_clause 函数**

在 `crates/parser/src/parser.rs` 第 1135 行附近找到 `parse_join_clause` 函数。

- [ ] **Step 2: 添加 Token::Full 处理分支**

在 `let join_type = match self.current()` 的 match 分支中添加：

```rust
Some(Token::Full) => {
    self.next();
    // FULL [OUTER] JOIN - consume optional OUTER
    if matches!(self.current(), Some(Token::Outer)) {
        self.next();
    }
    JoinType::Full
}
```

放置位置：在 `Token::Right` 分支之后，`Token::Cross` 分支之前。

- [ ] **Step 3: 验证语法解析**

运行测试验证：
```bash
cargo test -p sqlrustgo-parser -- test_parse_join
```

- [ ] **Step 4: 提交**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): handle FULL OUTER JOIN syntax"
```

---

## 3. Task 2: Executor 层 - FULL OUTER JOIN 实现

**Files:**
- Modify: `crates/executor/src/local_executor.rs:648-726` (execute_hash_join match 分支)

- [ ] **Step 1: 定位 execute_hash_join 中的 match join_type**

在 `crates/executor/src/local_executor.rs` 第 648 行附近找到 `match join_type` 代码块。

- [ ] **Step 2: 在 JoinType::Left 分支后添加 JoinType::Full 分支**

在 `_ => { Ok(ExecutorResult::empty()) }` 之前添加：

```rust
JoinType::Full => {
    use std::collections::HashSet;

    // Phase 1: Standard LEFT JOIN
    let matched = hash_inner_join(
        &left_result.rows,
        &right_result.rows,
        condition,
        left_schema,
        right_schema,
    );

    // Build hash set of matched right keys for deduplication
    let matched_right_keys: HashSet<Vec<Value>> = matched
        .iter()
        .skip(left_schema.fields.len())
        .cloned()
        .collect();

    // Phase 2: LEFT-only rows (same logic as LEFT JOIN)
    let left_only: Vec<Vec<Value>> = left_result
        .rows
        .iter()
        .filter(|lrow| {
            !matched.iter().any(|m| {
                m.iter().take(lrow.len()).cloned().collect::<Vec<_>>()
                    == lrow.iter().cloned().collect::<Vec<_>>()
            })
        })
        .map(|lrow| {
            let mut row = lrow.clone();
            row.extend(vec![Value::Null; right_schema.fields.len()]);
            row
        })
        .collect();

    // Phase 3: RIGHT-only rows (not in matched_right_keys)
    let right_only: Vec<Vec<Value>> = right_result
        .rows
        .iter()
        .filter(|rrow| {
            let key: Vec<Value> = rrow.iter().cloned().collect();
            !matched_right_keys.contains(&key)
        })
        .map(|rrow| {
            let mut row = vec![Value::Null; left_schema.fields.len()];
            row.extend(rrow.clone());
            row
        })
        .collect();

    // Combine all three parts
    let mut results = matched;
    results.extend(left_only);
    results.extend(right_only);

    let row_count = results.len();
    let duration = start.elapsed();

    GLOBAL_PROFILER.record(
        "HashJoin",
        "full_outer_join",
        duration.as_nanos() as u64,
        row_count,
        1,
    );

    Ok(ExecutorResult::new(results, 0))
}
```

- [ ] **Step 3: 验证编译**

```bash
cargo build -p sqlrustgo-executor --all-features 2>&1
```

- [ ] **Step 4: 提交**

```bash
git add crates/executor/src/local_executor.rs
git commit -m "feat(executor): implement FULL OUTER JOIN with 3-phase dedup"
```

---

## 4. Task 3: Parallel Executor 支持

**Files:**
- Modify: `crates/executor/src/parallel_executor.rs:310-340` (execute_hash_join match 分支)

- [ ] **Step 1: 定位 parallel_executor 中的 match join_type**

在 `crates/executor/src/parallel_executor.rs` 第 310 行附近找到相关代码。

- [ ] **Step 2: 添加 JoinType::Full 分支**

在现有的 `JoinType::Left | JoinType::LeftSemi | JoinType::LeftAnti` 处理后添加 `JoinType::Full` 分支。

```rust
JoinType::Full => {
    // Full outer join requires collecting all unmatched right rows
    // Split into: matched rows + left-only rows + right-only rows
    let all_matched = hash_inner_join(
        &left_result.rows,
        &right_result.rows,
        condition,
        left_schema,
        right_schema,
    );

    let matched_keys: HashSet<Vec<Value>> = all_matched
        .iter()
        .skip(left_schema.fields.len())
        .cloned()
        .collect();

    let left_only: Vec<Vec<Value>> = left_result
        .rows
        .iter()
        .filter(|lrow| {
            !all_matched.iter().any(|m| {
                m.iter().take(lrow.len()).cloned().collect::<Vec<_>>()
                    == lrow.iter().cloned().collect::<Vec<_>>()
            })
        })
        .map(|lrow| {
            let mut row = lrow.clone();
            row.extend(vec![Value::Null; right_schema.fields.len()]);
            row
        })
        .collect();

    let right_only: Vec<Vec<Value>> = right_result
        .rows
        .iter()
        .filter(|rrow| {
            let key: Vec<Value> = rrow.iter().cloned().collect();
            !matched_keys.contains(&key)
        })
        .map(|rrow| {
            let mut row = vec![Value::Null; left_schema.fields.len()];
            row.extend(rrow.clone());
            row
        })
        .collect();

    let mut results = all_matched;
    results.extend(left_only);
    results.extend(right_only);
    results
}
```

- [ ] **Step 3: 验证编译**

```bash
cargo build -p sqlrustgo-executor --all-features 2>&1
```

- [ ] **Step 4: 提交**

```bash
git add crates/executor/src/parallel_executor.rs
git commit -m "feat(executor): add FULL OUTER JOIN support to parallel executor"
```

---

## 5. Task 4: 单元测试

**Files:**
- Create: `crates/executor/tests/full_outer_join_test.rs`

- [ ] **Step 1: 创建测试文件**

```rust
use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_full_outer_join_basic() {
    let mut engine = create_engine();

    engine.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER, value INTEGER)").unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (1, 100), (2, 200), (4, 400)").unwrap();

    let result = engine.execute(
        "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id"
    ).unwrap();

    // Should have 4 rows: (1,a,1,100), (2,b,2,200), (3,c,NULL,NULL), (NULL,NULL,4,400)
    assert_eq!(result.rows.len(), 4);
}

#[test]
fn test_full_outer_join_all_match() {
    let mut engine = create_engine();

    engine.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER, value INTEGER)").unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (1, 100), (2, 200)").unwrap();

    let result = engine.execute(
        "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id"
    ).unwrap();

    // Should have 2 rows, all matched
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_full_outer_join_no_match() {
    let mut engine = create_engine();

    engine.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER, value INTEGER)").unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (3, 100), (4, 200)").unwrap();

    let result = engine.execute(
        "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id"
    ).unwrap();

    // Should have 4 rows, none matched
    assert_eq!(result.rows.len(), 4);
}

#[test]
fn test_full_outer_join_with_nulls() {
    let mut engine = create_engine();

    engine.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER, value INTEGER)").unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 'a'), (NULL, 'b')").unwrap();
    engine.execute("INSERT INTO t2 VALUES (1, 100), (NULL, 200)").unwrap();

    let result = engine.execute(
        "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id"
    ).unwrap();

    // NULLs should not match each other
    assert_eq!(result.rows.len(), 3);
}
```

- [ ] **Step 2: 运行测试验证**

```bash
cargo test -p sqlrustgo-executor --test full_outer_join_test -- --nocapture 2>&1
```

- [ ] **Step 3: 提交**

```bash
git add crates/executor/tests/full_outer_join_test.rs
git commit -m "test: add FULL OUTER JOIN unit tests"
```

---

## 6. Task 5: 集成测试 - TPC-DS Q2.6

**Files:**
- Modify: `tests/` (现有集成测试文件)

- [ ] **Step 1: 找到 TPC-DS 测试文件**

```bash
ls tests/e2e/ | grep -i tpc
# 或
ls tests/ | grep -i tpc
```

- [ ] **Step 2: 添加 TPC-DS Q2.6 测试**

如果存在 TPC-DS 测试文件，添加 Q2.6 测试。如果不存在，创建新文件。

TPC-DS Q2.6 近似查询：
```sql
-- Q2.6: FULL OUTER JOIN between store sales and catalog sales
SELECT s.store_id, s.amount_sold, c.catalog_id, c.amount_sold
FROM store_sales s
FULL OUTER JOIN catalog_sales c ON s.date_id = c.date_id AND s.item_id = c.item_id
WHERE s.date_id BETWEEN '1999-01-01' AND '1999-12-31'
```

- [ ] **Step 3: 运行集成测试**

```bash
cargo test --test <tpcds_test_file> 2>&1
```

- [ ] **Step 4: 提交**

```bash
git add tests/
git commit -m "test: add TPC-DS Q2.6 FULL OUTER JOIN test"
```

---

## 7. Task 6: 最终验证

- [ ] **Step 1: 运行完整测试套件**

```bash
cargo test --all-features 2>&1 | tail -50
```

- [ ] **Step 2: 运行 clippy**

```bash
cargo clippy --all-features -- -D warnings 2>&1
```

- [ ] **Step 3: 格式化检查**

```bash
cargo fmt --check --all 2>&1
```

- [ ] **Step 4: 合并所有更改**

```bash
git add -A
git commit -m "feat: implement FULL OUTER JOIN with HashJoin
- Parser: handle FULL [OUTER] JOIN syntax
- Executor: 3-phase dedup algorithm
- Tests: unit tests + TPC-DS Q2.6"
```

---

## 自检清单

- [ ] Parser `Token::Full` 处理已添加
- [ ] `JoinType::Full` 在 local_executor 中实现
- [ ] `JoinType::Full` 在 parallel_executor 中实现
- [ ] 单元测试覆盖基本场景
- [ ] TPC-DS Q2.6 测试通过
- [ ] Clippy 无警告
- [ ] 所有测试通过

---

## 执行选项

**Plan complete and saved to `docs/superpowers/plans/2026-04-20-full-outer-join-plan.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
