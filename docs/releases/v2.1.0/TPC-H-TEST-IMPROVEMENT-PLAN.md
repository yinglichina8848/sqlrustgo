# TPC-H 测试执行改进计划

**版本**: v2.1.0
**日期**: 2026-04-02
**状态**: 🚨 需要修复
**优先级**: P0

---

## 一、问题概述

### 1.1 核心问题：MockStorage 是 Stub

**位置**: `crates/executor/src/mock_storage.rs:169-179`

```rust
fn delete(&mut self, _table: &str, _filters: &[Value]) -> SqlResult<usize> {
    Ok(0)  // ❌ 总是返回 0!
}

fn update(&mut self, _table: &str, _filters: &[Value], _updates: &[(usize, Value)]) -> SqlResult<usize> {
    Ok(0)  // ❌ 总是返回 0!
}
```

### 1.2 影响范围

| 影响项 | 严重程度 | 说明 |
|--------|----------|------|
| TPC-H 测试 | 🚨 严重 | 无法验证 UPDATE/DELETE |
| FK 约束测试 | 🚨 严重 | delete/update 操作假执行 |
| 集成测试 | 🚨 严重 | 使用 MockStorage 的测试无法验证 DML |

### 1.3 当前测试存储使用

| 测试文件 | 存储类型 | DML 测试 |
|----------|----------|----------|
| `tpch_test.rs` | MockStorage | ❌ 无法测试 |
| `tpch_benchmark.rs` | MockStorage | ❌ 无法测试 |
| `foreign_key_test.rs` | MemoryStorage | ✅ 真实执行 |
| `upsert_test.rs` | MemoryStorage | ✅ 真实执行 |

---

## 二、执行路径分析

### 2.1 两条 DML 执行路径

#### 路径 A: ExecutionEngine (✅ 真实)
```rust
// src/lib.rs:1127-1230
Statement::Delete(delete) => {
    // 1. 扫描所有行
    // 2. 评估 WHERE 子句
    // 3. 处理 FK 约束
    // 4. 删除匹配行
}
Statement::Update(update) => {
    // 真实实现，包含 FK 约束处理
}
```

#### 路径 B: sql-cli (❌ 不支持)
```rust
// crates/sql-cli/src/main.rs:176
_ => Err("Only SELECT, INSERT, CREATE TABLE, DROP TABLE..."),
```

### 2.2 问题

1. **sql-cli 不支持 UPDATE/DELETE** - 用户无法通过 CLI 使用
2. **MockStorage delete/update 是 stub** - 测试使用时不验证 DML
3. **TPC-H 测试用 MockStorage** - 假执行

---

## 三、改进计划

### 3.1 Phase 1: MockStorage 修复 (Week 1)

#### Task 1.1: 实现 MockStorage::delete

**目标**: 让 MockStorage::delete 返回实际删除的行数

**当前代码**:
```rust
fn delete(&mut self, _table: &str, _filters: &[Value]) -> SqlResult<usize> {
    Ok(0)  // stub
}
```

**期望代码**:
```rust
fn delete(&mut self, table: &str, filters: &[Value]) -> SqlResult<usize> {
    let mut tables = self.tables.write().unwrap();
    let table_data = tables.get_mut(table).ok_or_else(|| 
        SqlError::TableNotFound { table: table.to_string() }
    )?;
    
    // 如果 filters 为空，删除所有行
    if filters.is_empty() {
        let count = table_data.len();
        table_data.clear();
        return Ok(count);
    }
    
    // 解析 filters: [column_index, value]
    // 或只有一个 value，匹配 column 0
    let (col_idx, match_val) = if filters.len() >= 2 {
        match &filters[0] {
            Value::Integer(i) => (*i as usize, &filters[1]),
            _ => return Err(SqlError::ExecutionError("Filter index must be integer".to_string())),
        }
    } else {
        (0, &filters[0])
    };
    
    // 过滤并删除匹配行
    let original_len = table_data.len();
    table_data.retain(|row| {
        row.get(col_idx).map(|v| v != match_val).unwrap_or(true)
    });
    
    Ok(original_len - table_data.len())
}
```

**验收**: `cargo test mock_storage_delete -- --nocapture`

#### Task 1.2: 实现 MockStorage::update

**目标**: 让 MockStorage::update 返回实际更新的行数

**当前代码**:
```rust
fn update(&mut self, _table: &str, _filters: &[Value], _updates: &[(usize, Value)]) -> SqlResult<usize> {
    Ok(0)  // stub
}
```

**期望代码**:
```rust
fn update(&mut self, table: &str, filters: &[Value], updates: &[(usize, Value)]) -> SqlResult<usize> {
    let mut tables = self.tables.write().unwrap();
    let table_data = tables.get_mut(table).ok_or_else(|| 
        SqlError::TableNotFound { table: table.to_string() }
    )?;
    
    if updates.is_empty() {
        return Ok(0);
    }
    
    // 解析 filters
    let (col_idx, match_val) = if filters.len() >= 2 {
        match &filters[0] {
            Value::Integer(i) => (*i as usize, &filters[1]),
            _ => return Err(SqlError::ExecutionError("Filter index must be integer".to_string())),
        }
    } else {
        (0, &filters[0])
    };
    
    let mut count = 0;
    for row in table_data.iter_mut() {
        if row.get(col_idx).map(|v| v == match_val).unwrap_or(false) {
            for (update_col, new_val) in updates {
                if *update_col < row.len() {
                    row[*update_col] = new_val.clone();
                }
            }
            count += 1;
        }
    }
    
    Ok(count)
}
```

**验收**: `cargo test mock_storage_update -- --nocapture`

#### Task 1.3: 实现 MockStorage 其他缺失方法

| 方法 | 当前 | 期望 |
|------|------|------|
| create_index | Ok(()) | 构建 B+Tree 索引 |
| drop_index | Ok(()) | 删除索引 |
| search_index | None | 返回索引搜索结果 |
| range_index | vec![] | 返回范围搜索结果 |

---

### 3.2 Phase 2: TPC-H 测试框架改进 (Week 1-2)

#### Task 2.1: 识别 TPC-H 测试中的假执行

**目标**: 找出所有只打印 `assert!(true)` 的测试

```bash
grep -n "assert!(true)" tests/integration/tpch_full_test.rs
```

**问题测试** (tpch_full_test.rs):
- `test_tpch_q1_pricing_summary` - 行 30
- `test_tpch_q2_minimum_cost_supplier` - 行 40
- ... 所有 28 个测试都是 stub

#### Task 2.2: 实现真实的 TPC-H Q1 测试

**目标**: 使用真实数据执行 Q1

**当前** (行 20-31):
```rust
fn test_tpch_q1_pricing_summary() {
    println!("\n=== TPC-H Q1 ===");
    let start = Instant::now();
    // 模拟执行 - 假!
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs_f64() < 1.0);
}
```

**期望**:
```rust
fn test_tpch_q1_pricing_summary() {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
    use std::sync::{Arc, RwLock};
    
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    
    // 创建 lineitem 表
    engine.execute(parse(
        "CREATE TABLE lineitem (
            l_orderkey INTEGER,
            l_partkey INTEGER,
            l_suppkey INTEGER,
            l_linenumber INTEGER,
            l_quantity INTEGER,
            l_extendedprice REAL,
            l_discount REAL,
            l_tax REAL,
            l_returnflag TEXT,
            l_linestatus TEXT,
            l_shipdate TEXT,
            l_commitdate TEXT,
            l_receiptdate TEXT,
            l_shipinstruct TEXT,
            l_shipmode TEXT,
            l_comment TEXT
        )"
    ).unwrap()).unwrap();
    
    // 插入测试数据
    engine.execute(parse(
        "INSERT INTO lineitem VALUES 
        (1, 1, 1, 1, 17, 1000.0, 0.06, 80.0, 'N', 'O', '1998-12-01', '1998-12-01', '1998-12-02', 'DELIVER IN PERSON', 'TRUCK', 'test')"
    ).unwrap()).unwrap();
    
    // 执行 Q1
    let result = engine.execute(parse(
        "SELECT l_returnflag, l_linestatus,
                SUM(l_quantity) AS sum_qty,
                SUM(l_extendedprice) AS sum_base_price
         FROM lineitem
         WHERE l_shipdate <= '1998-12-01'
         GROUP BY l_returnflag, l_linestatus
         ORDER BY l_returnflag, l_linestatus"
    ).unwrap()).unwrap();
    
    assert!(!result.rows.is_empty());
}
```

#### Task 2.3: 批量实现 Q2-Q22 测试

按依赖关系排序实现:

| 阶段 | 查询 | 依赖 |
|------|------|------|
| 1 | Q1, Q6 | 简单过滤 + 聚合 |
| 2 | Q3, Q4, Q5 | DATE 过滤 (依赖 DATE 解析) |
| 3 | Q2, Q7, Q10 | JOIN |
| 4 | Q12, Q14, Q16, Q19 | IN, CASE (依赖语法支持) |

**注意**: Q3, Q4, Q5 等依赖 DATE 解析，与另一个 AI 的 Phase 1 并行

---

### 3.3 Phase 3: sql-cli DML 支持 (Week 2)

#### Task 3.1: 在 sql-cli 中添加 UPDATE 支持

**位置**: `crates/sql-cli/src/main.rs:162-177`

**当前**:
```rust
match statement {
    Statement::Select(select) => execute_select(&select, storage),
    Statement::Insert(insert) => execute_insert(&insert, storage),
    Statement::CreateTable(create) => execute_create_table(&create, storage),
    Statement::DropTable(drop) => execute_drop_table(&drop, storage),
    _ => Err("Only SELECT, INSERT, CREATE TABLE, DROP TABLE..."),
}
```

**期望**:
```rust
match statement {
    Statement::Select(select) => execute_select(&select, storage),
    Statement::Insert(insert) => execute_insert(&insert, storage),
    Statement::Update(update) => execute_update(&update, storage),
    Statement::Delete(delete) => execute_delete(&delete, storage),
    Statement::CreateTable(create) => execute_create_table(&create, storage),
    Statement::DropTable(drop) => execute_drop_table(&drop, storage),
    _ => Err("Unsupported statement type"),
}
```

#### Task 3.2: 实现 execute_update 函数

```rust
fn execute_update(
    update: &UpdateStatement,
    storage: &dyn StorageEngine,
) -> Result<ExecutorResult, String> {
    let mut storage = storage.write().unwrap();
    
    if !storage.has_table(&update.table) {
        return Err(format!("Table '{}' not found", update.table));
    }
    
    let table_info = storage.get_table_info(&update.table).ok()
        .ok_or_else(|| format!("Table '{}' not found", update.table))?;
    
    let columns = table_info.columns;
    let all_rows = storage.scan(&update.table).map_err(|e| e.to_string())?;
    
    let mut updated_count = 0;
    let mut new_rows = Vec::new();
    
    for row in all_rows {
        let should_update = update.where_clause.as_ref()
            .map(|wc| evaluate_where_clause(wc, &row, &columns))
            .unwrap_or(true);
        
        if should_update {
            let mut new_row = row.clone();
            for (col_name, expr) in &update.set_clauses {
                if let Some(col_idx) = columns.iter().position(|c| c.name == *col_name) {
                    let value = evaluate_expression(expr, &row);
                    new_row[col_idx] = value;
                }
            }
            new_rows.push(new_row);
            updated_count += 1;
        } else {
            new_rows.push(row);
        }
    }
    
    storage.delete(&update.table, &[]).map_err(|e| e.to_string())?;
    storage.insert(&update.table, new_rows).map_err(|e| e.to_string())?;
    
    Ok(ExecutorResult::new(vec![], updated_count))
}
```

#### Task 3.3: 实现 execute_delete 函数

类似 UPDATE 实现

---

### 3.4 Phase 4: 回归测试确保 (Week 2-3)

#### Task 4.1: 确保所有测试使用真实存储

**目标**: 将使用 MockStorage 的测试改为 MemoryStorage

```bash
# 查找使用 MockStorage 的测试
grep -rn "MockStorage" tests/ | grep -v "test_framework"
```

**需要修改的测试**:
- `tests/integration/tpch_test.rs` - 改为 MemoryStorage
- `tests/integration/tpch_benchmark.rs` - 改为 MemoryStorage

#### Task 4.2: 添加 DML 测试覆盖

**目标**: 确保 UPDATE/DELETE 有真实测试

```rust
#[test]
fn test_update_with_where_clause() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    // ... setup ...
    
    let result = engine.execute(parse(
        "UPDATE users SET name = 'Updated' WHERE id = 1"
    ).unwrap()).unwrap();
    
    assert_eq!(result.row_count, 1);  // 验证更新行数
}

#[test]
fn test_delete_with_where_clause() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    // ... setup ...
    
    let result = engine.execute(parse(
        "DELETE FROM users WHERE id = 1"
    ).unwrap()).unwrap();
    
    assert_eq!(result.row_count, 1);  // 验证删除行数
}
```

---

## 四、验收标准

### 4.1 Phase 1 (MockStorage 修复)

| 测试 | 标准 |
|------|------|
| `cargo test mock_storage_delete` | 返回实际删除行数 |
| `cargo test mock_storage_update` | 返回实际更新行数 |
| `cargo test mock_storage_index` | 索引创建和搜索工作 |

### 4.2 Phase 2 (TPC-H 测试)

| 测试 | 标准 |
|------|------|
| `cargo test --test tpch_full_test test_tpch_q1` | Q1 执行并返回结果 |
| `cargo test --test tpch_full_test test_tpch_q6` | Q6 执行并返回结果 |
| `cargo test --test tpch_full_test` | Q1-Q22 全部可执行 |

### 4.3 Phase 3 (sql-cli)

| 测试 | 标准 |
|------|------|
| `echo "UPDATE ..." \| cargo run --bin sql-cli` | 执行成功 |
| `echo "DELETE ..." \| cargo run --bin sql-cli` | 执行成功 |

### 4.4 Phase 4 (回归)

| 测试 | 标准 |
|------|------|
| `cargo test --test regression_test` | 全部通过 |
| `cargo test --test foreign_key_test` | FK 约束测试通过 |

---

## 五、与其他 AI 计划的协调

### 5.1 分工

| AI | 负责范围 |
|-----|----------|
| **本计划** | MockStorage 修复、TPC-H 测试框架、sql-cli DML |
| **另一个 AI** | BETWEEN、DATE、IN、CASE 语法实现 |

### 5.2 并行工作

两个计划可以并行执行，因为:
- 语法实现 (BETWEEN/DATE/IN/CASE) 在 Parser 层
- MockStorage 修复在 Executor/Storage 层
- 没有依赖冲突

### 5.3 依赖关系

- TPC-H Q1-Q5 依赖 DATE 解析 (另一个 AI 负责)
- TPC-H Q6 不依赖语法 (可以直接实现)
- TPC-H Q12/Q16 依赖 IN/CASE (另一个 AI 负责)

---

## 六、风险和缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| MockStorage 修改复杂 | 高 | 参考 MemoryStorage 实现 |
| TPC-H 测试数据量大 | 中 | 使用简化数据集 |
| 与另一个 AI 冲突 | 低 | 分工明确，定期同步 |

---

## 七、进度追踪

| Phase | Task | 状态 | 完成日期 |
|-------|------|------|----------|
| 1.1 | MockStorage::delete | ⏳ | - |
| 1.2 | MockStorage::update | ⏳ | - |
| 1.3 | MockStorage 索引方法 | ⏳ | - |
| 2.1 | 识别假执行测试 | ✅ | 2026-04-02 |
| 2.2 | 实现 Q1 测试 | ⏳ | - |
| 3.1 | sql-cli UPDATE | ⏳ | - |
| 3.2 | sql-cli DELETE | ⏳ | - |
| 4.1 | 测试存储类型统一 | ⏳ | - |

---

*计划创建: 2026-04-02*
*最后更新: 2026-04-02*
