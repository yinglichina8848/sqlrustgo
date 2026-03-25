# SQLRustGo vs SQLite 性能分析报告

## 测试结果汇总

| 场景 | SF 0.1 | SF 1.0 | 优势方 |
|------|--------|--------|--------|
| Q1 全表扫描 | 14x慢 | 2.1x慢 | SQLite |
| Q3 Join | 85x慢 | 7.7x慢 | SQLite |
| Q6 谓词下推 | 4x慢 | **3.27x快** | SQLRustGo |
| Q10 复杂Join | 130x慢 | 13.6x慢 | SQLite |

## 性能差异根本原因分析

### 1. Q6 谓词下推 - SQLRustGo 胜出 (3.27x)

**原因：**
```
SQLRustGo: 存储层直接过滤 → 只读取满足条件的数据
SQLite:    先读取全表 → 再在执行层过滤
```

**代码证据：**
- `crates/storage/src/filter.rs` 实现了存储级过滤
- 谓词下推在数据读取前执行，减少 I/O

### 2. Q1/Q3/Q10 Join 操作 - SQLite 优势

**原因分析：**

| 因素 | SQLite | SQLRustGo |
|------|--------|-----------|
| Join 算法 | 优化多年的嵌套循环+哈希 | 简单哈希join |
| 查询计划 | 基于代价优化器选择最优计划 | 规则优化器 |
| 索引利用 | 智能使用索引做 join | 索引使用不充分 |
| 并行执行 | 多线程执行 | 有限并行 |
| 预编译 | SQL 预编译缓存 | 无预编译 |

**关键代码问题 (executor.rs:474-492):**

```rust
// 问题1: 一次性加载所有右表数据到内存
while let Some(row) = self.right.next()? {
    self.right_hash.entry(key).or_default().push(row);
}

// 问题2: 同样一次性加载所有左表
while let Some(row) = self.left.next()? {
    self.current_left_rows.push(row);
}
```

这导致：
1. 大数据集内存压力
2. 无流式处理
3. 无法利用索引

## 优化方案

### 高优先级 (预计提升 3-5x)

#### 1. 实现 Join 重排序
```rust
// 根据统计信息自动选择最优 join 顺序
fn reorder_joins(plan: &LogicalPlan, stats: &Statistics) -> LogicalPlan
```

#### 2. 添加查询计划缓存
```rust
// 编译后的查询计划缓存
pub struct QueryPlanCache {
    plans: DashMap<u64, CompiledPlan>,
}
```

#### 3. 改进 Hash Join - 分批流式处理
```rust
// 当前：全量加载
while let Some(row) = self.right.next()? { ... }

// 改进：分批处理 + 溢出到磁盘
fn next_batch(&mut self) -> SqlResult<Option<RecordBatch>> {
    let mut batch = self.right.read_batch(1024)?;
    for row in batch {
        self.build_hash(&row)?;
    }
}
```

### 中优先级 (预计提升 1.5-2x)

#### 4. 布隆过滤器 Join
```rust
// 用布隆过滤器快速过滤不匹配的右表数据
fn apply_bloom_filter(&self, build: &HashTable, probe: &Vec<Value>) -> Vec<bool>
```

#### 5. 向量化执行
```rust
// 当前：逐行处理
fn next(&mut self) -> SqlResult<Option<Vec<Value>>>

// 改进：批量处理
fn next_batch(&mut self) -> SqlResult<Option<RecordBatch>>
```

#### 6. 并行执行
```rust
// 多线程并行扫描和 join
fn parallel_hash_join(left: &Data, right: &Data, num_threads: 4) 
```

### 低优先级

#### 7. 列式存储支持
#### 8. 自适应查询执行
#### 9. 物化视图缓存

## 预期优化效果

| 优化项 | Q1 | Q3 | Q6 | Q10 |
|--------|-----|-----|-----|------|
| Join 重排序 | - | 2x | - | 2x |
| 计划缓存 | 1.5x | 1.5x | 1.5x | 1.5x |
| 流式 Hash Join | - | 3x | - | 3x |
| 向量化 | 1.5x | 1.5x | 1.5x | 1.5x |
| 并行执行 | 2x | 2x | - | 2x |
| **综合** | **~4x** | **~10x** | **~2x** | **~10x** |

## 结论

1. **SQLRustGo 优势场景**：谓词下推查询 (Q6) 已优于 SQLite
2. **主要瓶颈**：Join 操作的算法和实现
3. **建议路线**：先实现流式 Hash Join + Join 重排序，预计可提升 5-10x
