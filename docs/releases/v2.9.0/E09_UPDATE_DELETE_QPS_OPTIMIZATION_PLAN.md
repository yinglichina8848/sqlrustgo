# E-09 UPDATE/DELETE QPS 优化实施计划

## Issue
Issue #296: E-09 UPDATE/DELETE QPS 优化

## 背景

根据 E-08 QPS 基准测试报告，UPDATE/DELETE 操作严重不达标：

| 操作 | 目标 QPS | 实际 QPS | 达成率 |
|------|----------|----------|--------|
| UPDATE | ≥10,000 | ~950 | 9.5% |
| DELETE | ≥10,000 | ~206 | 2% |

## 根因分析

### 1. 表达式评估开销（已部分解决）

`evaluate_sql_expression` 函数（`crates/storage/src/engine.rs:54-130`）对每个条件表达式进行字符串解析：
- `to_uppercase()` 在每个操作数上调用
- `find_top_level_op()` 遍历整个字符串
- 递归解析 AND/OR 表达式

**已解决**：DELETE 路径已改用 AST 评估（`eval_predicate`），避免字符串解析。

### 2. 双重扫描问题（待解决）

`execute_update` 中存在双重表扫描：

```rust
// 第 1 次扫描 - 获取所有行
let all_rows = {
    let storage = self.storage.read().unwrap();
    storage.scan(&table_name)?;  // ← 第一次扫描
};

// ... 处理匹配行 ...

// 第 2 次扫描 - 获取 rows_to_keep
let rows_to_keep: Vec<Vec<Value>> = {
    let storage = self.storage.read().unwrap();
    let all_rows = storage.scan(&table_name)?;  // ← 第二次扫描！
    all_rows.into_iter().filter(...).collect()
};
```

类似问题存在于 DELETE 路径。

### 3. Delete-All + Reinsert 反模式（待解决）

即使只删除/更新 1 行，当前实现也会：
1. 删除表中所有行
2. 重新插入未匹配的行

这是最严重的性能瓶颈。

## 优化方案

### 方案 4: 表达式缓存（优先级 1）

**原理**：将字符串表达式解析结果缓存，避免重复解析。

**改动范围**：
- `crates/storage/src/engine.rs` - 添加缓存逻辑

**实现**：
```rust
// 在 MemoryStorage 中添加
use std::collections::HashMap;
use std::sync::Mutex;

struct ParsedExpr {
    pub columns: Vec<String>,
    pub ast: Expr,  // 预解析的 AST
}

impl MemoryStorage {
    fn get_or_parse_predicate(&mut self, expr: &str) -> SqlResult<ParsedExpr> {
        // 检查缓存
        if let Some(cached) = self.predicate_cache.lock().unwrap().get(expr) {
            return Ok(cached.clone());
        }
        // 解析并缓存
        let parsed = self.parse_predicate(expr)?;
        self.predicate_cache.lock().unwrap().insert(expr.to_string(), parsed.clone());
        Ok(parsed)
    }
}
```

**预估时间**：0.5-1 天
**预估效果**：约 1.5-2x 提升

### 方案 2: 消除双重扫描（优先级 2）

**原理**：单次扫描时同时记录匹配/不匹配的行，避免重复扫描。

**改动范围**：
- `src/execution_engine.rs` - 修改 `execute_update` 和 `execute_delete`

**实现**：

```rust
// 单次扫描，同时收集匹配索引和不匹配数据
let (rows_to_update_indices, rows_to_keep): (Vec<usize>, Vec<Vec<Value>>) = {
    let mut indices = Vec::new();
    let mut keep = Vec::new();
    for (i, row) in all_rows.iter().enumerate() {
        if evaluate_where_clause(where_clause, row, &table_info) {
            indices.push(i);
        } else {
            keep.push(row.clone());
        }
    }
    (indices, keep)
};
```

**预估时间**：1-2 天
**预估效果**：约 2-3x 提升

### 方案 1: 原位删除/更新（优先级 3）

**原理**：不 delete-all + reinsert，直接在 Vec 中删除或修改指定索引的行。

**改动范围**：

| 文件 | 修改内容 | 难度 |
|------|----------|------|
| `crates/storage/src/engine.rs` | trait 添加 `delete_by_indices`, `update_by_indices` | 中 |
| `crates/storage/src/memory_storage.rs` | 实现新方法 | 低 |
| `crates/storage/src/file_storage.rs` | 实现新方法（处理页面和索引） | 高 |
| `crates/storage/src/columnar/storage.rs` | 实现新方法 | 高 |
| `src/execution_engine.rs` | 调用新方法 | 低 |

**实现示例（MemoryStorage）**：

```rust
fn delete_by_indices(&mut self, table: &str, indices: &[usize]) -> SqlResult<usize> {
    let records = self.tables.get_mut(table).ok_or_else(|| ...)?;
    let count = indices.len();

    // 按索引倒序删除，避免删除后索引偏移
    let mut sorted_indices = indices.to_vec();
    sorted_indices.sort_unstable();
    sorted_indices.reverse();

    for idx in sorted_indices {
        records.remove(idx);
    }
    Ok(count)
}
```

**预估时间**：1-2 周
**预估效果**：约 10-20x 提升，达到 ≥10,000 QPS 目标

## 实施顺序

1. **方案 4**（0.5-1 天）：表达式缓存
2. **方案 2**（1-2 天）：消除双重扫描
3. **方案 1**（1-2 周）：原位删除/更新

## 预期效果

| 阶段 | UPDATE QPS | DELETE QPS |
|------|------------|------------|
| 当前基线 | ~1,200 | ~500 |
| 方案 4 后 | ~2,000 | ~1,000 |
| 方案 2 后 | ~4,000 | ~2,500 |
| 方案 1 后 | ≥10,000 | ≥10,000 |

## 风险与注意事项

### 方案 1 的风险

1. **ColumnarStorage 还未实现 DELETE/UPDATE**
   - 当前返回 "not yet implemented" 错误
   - 新方法需要考虑与 ColumnarStorage 的兼容性

2. **FileStorage 需要处理 B+Tree 索引**
   - 删除行后需要更新索引
   - 可能需要引入"墓碑"机制

3. **MVCC 事务支持**
   - MVCCStorage 包装了其他存储
   - 新方法需要考虑版本控制

### 测试要求

每个方案实施后需要验证：
1. `cargo test --all-features` 全部通过
2. QPS 基准测试达标
3. 与 SQLite 对比测试通过

## 相关文档

- E-08 QPS 基准测试报告：`docs/releases/v2.9.0/E08_QPS_BENCHMARK_REPORT.md`
- Hash Join 优化 PR：#299
- 存储引擎设计：`crates/storage/src/engine.rs`
- 执行引擎：`src/execution_engine.rs`
