# MockStorage Stub 修复计划

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
| DML 测试 | 🚨 严重 | 无法验证 UPDATE/DELETE 实际行数 |
| FK 约束测试 | 🚨 严重 | delete/update 操作假执行 |
| sql-cli | 🚨 严重 | 不支持 UPDATE/DELETE 命令 |

---

## 二、执行路径分析

### 2.1 两条 DML 执行路径

#### 路径 A: ExecutionEngine (✅ 真实)
```rust
// src/lib.rs:1127-1230
Statement::Delete(delete) => { /* 真实实现 */ }
Statement::Update(update) => { /* 真实实现 */ }
```

#### 路径 B: sql-cli (❌ 不支持)
```rust
// crates/sql-cli/src/main.rs:176
_ => Err("Only SELECT, INSERT, CREATE TABLE, DROP TABLE..."),
```

---

## 三、改进计划

### Phase 1: MockStorage 修复 (Week 1)

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
    
    if filters.is_empty() {
        let count = table_data.len();
        table_data.clear();
        return Ok(count);
    }
    
    let (col_idx, match_val) = if filters.len() >= 2 {
        match &filters[0] {
            Value::Integer(i) => (*i as usize, &filters[1]),
            _ => return Err(SqlError::ExecutionError("Filter index must be integer".to_string())),
        }
    } else {
        (0, &filters[0])
    };
    
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

### Phase 2: sql-cli DML 支持 (Week 2)

#### Task 2.1: 添加 UPDATE/DELETE 到 sql-cli

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

#### Task 2.2: 实现 execute_update 函数

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

#### Task 2.3: 实现 execute_delete 函数

类似 UPDATE 实现

---

### Phase 3: 回归测试确保 (Week 2-3)

#### Task 3.1: 确保所有测试使用真实存储

```bash
# 查找使用 MockStorage 的测试
grep -rn "MockStorage" tests/ | grep -v "test_framework"
```

**需要修改的测试**:
- `tests/integration/tpch_test.rs` - 改为 MemoryStorage
- `tests/integration/tpch_benchmark.rs` - 改为 MemoryStorage

#### Task 3.2: 添加 DML 测试覆盖

```rust
#[test]
fn test_update_with_where_clause() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    // ... setup ...
    
    let result = engine.execute(parse(
        "UPDATE users SET name = 'Updated' WHERE id = 1"
    ).unwrap()).unwrap();
    
    assert_eq!(result.row_count, 1);
}

#[test]
fn test_delete_with_where_clause() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    // ... setup ...
    
    let result = engine.execute(parse(
        "DELETE FROM users WHERE id = 1"
    ).unwrap()).unwrap();
    
    assert_eq!(result.row_count, 1);
}
```

---

## 四、验收标准

### Phase 1 (MockStorage)

| 测试 | 标准 |
|------|------|
| `cargo test mock_storage_delete` | 返回实际删除行数 |
| `cargo test mock_storage_update` | 返回实际更新行数 |
| `cargo test mock_storage_index` | 索引创建和搜索工作 |

### Phase 2 (sql-cli)

| 测试 | 标准 |
|------|------|
| `echo "UPDATE ..." \| cargo run --bin sql-cli` | 执行成功 |
| `echo "DELETE ..." \| cargo run --bin sql-cli` | 执行成功 |

### Phase 3 (回归)

| 测试 | 标准 |
|------|------|
| `cargo test --test regression_test` | 全部通过 |
| `cargo test --test foreign_key_test` | FK 约束测试通过 |

---

## 五、与另一个 AI 计划的协调

### 分工

| AI | 负责范围 |
|-----|----------|
| **本计划** | MockStorage 修复、sql-cli DML、回归测试 |
| **另一个 AI** | TPC-H 测试改进、BETWEEN、DATE、IN、CASE |

### 并行工作

两个计划可以并行执行，因为:
- 语法实现 (BETWEEN/DATE/IN/CASE) 在 Parser 层
- MockStorage 修复在 Executor/Storage 层
- 没有依赖冲突

---

## 六、进度追踪

| Phase | Task | 状态 | 完成日期 |
|-------|------|------|----------|
| 1.1 | MockStorage::delete | ⏳ | - |
| 1.2 | MockStorage::update | ⏳ | - |
| 1.3 | MockStorage 索引方法 | ⏳ | - |
| 2.1 | sql-cli UPDATE | ⏳ | - |
| 2.2 | sql-cli DELETE | ⏳ | - |
| 3.1 | 测试存储类型统一 | ⏳ | - |
| 3.2 | DML 测试覆盖 | ⏳ | - |

---

*计划创建: 2026-04-02*
*最后更新: 2026-04-02*
