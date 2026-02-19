# Join 算法演进路线

> Join Algorithm Evolution

---

## 阶段 1：Nested Loop Join（v1.0）

### 实现

```rust
struct NestedLoopJoin {
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    condition: Expr,
}

impl Executor for NestedLoopJoin {
    fn next(&mut self) -> Option<Tuple> {
        loop {
            if self.current_right.is_none() {
                self.current_left = self.left.next()?;
                self.right.reset();
                self.current_right = self.right.next();
            }

            if let Some(right_tuple) = &self.current_right {
                let left = self.current_left.as_ref()?;
                if evaluate(&self.condition, left, right_tuple) {
                    let result = merge(left, right_tuple);
                    self.current_right = self.right.next();
                    return Some(result);
                }
                self.current_right = self.right.next();
            }
        }
    }
}
```

### 复杂度

```
O(N × M)
```

- N = 左表行数
- M = 右表行数

### 适用场景

- 小表 join
- 无索引情况
- 简单实现

---

## 阶段 2：Hash Join（v2.0）

### 实现

```rust
struct HashJoin {
    build_side: Box<dyn Executor>,
    probe_side: Box<dyn Executor>,
    hash_table: HashMap<Key, Vec<Tuple>>,
    build_key: Expr,
    probe_key: Expr,
}

impl HashJoin {
    fn build(&mut self) {
        while let Some(tuple) = self.build_side.next() {
            let key = evaluate(&self.build_key, &tuple);
            self.hash_table.entry(key).or_default().push(tuple);
        }
    }

    fn probe(&mut self) -> Option<Tuple> {
        loop {
            if let Some(probe_tuple) = &self.current_probe {
                let key = evaluate(&self.probe_key, probe_tuple);
                if let Some(matches) = self.hash_table.get(&key) {
                    // 返回匹配的 tuple
                }
            }
            self.current_probe = self.probe_side.next();
        }
    }
}
```

### 复杂度

```
O(N + M)
```

### 适用场景

- 大表 join
- 等值连接
- 内存足够

### Spill 处理

当内存不足时：

```rust
if hash_table.memory() > memory_limit {
    // 分区
    let partitions = partition_by_hash(&hash_table);
    
    // 写入磁盘
    for partition in partitions {
        write_to_disk(partition);
    }
    
    // 递归处理
    for partition_file in partition_files {
        process_partition(partition_file);
    }
}
```

---

## 阶段 3：Sort-Merge Join（v2.x）

### 实现

```rust
struct SortMergeJoin {
    left_sorted: SortedIterator,
    right_sorted: SortedIterator,
    join_key: Expr,
}

impl Executor for SortMergeJoin {
    fn next(&mut self) -> Option<Tuple> {
        loop {
            match (self.left_current.as_ref(), self.right_current.as_ref()) {
                (Some(left), Some(right)) => {
                    let left_key = evaluate(&self.join_key, left);
                    let right_key = evaluate(&self.join_key, right);
                    
                    match left_key.cmp(&right_key) {
                        Ordering::Less => {
                            self.left_current = self.left_sorted.next();
                        }
                        Ordering::Greater => {
                            self.right_current = self.right_sorted.next();
                        }
                        Ordering::Equal => {
                            let result = merge(left, right);
                            // 处理相同 key 的多个 tuple
                            return Some(result);
                        }
                    }
                }
                _ => return None,
            }
        }
    }
}
```

### 复杂度

```
O(N log N + M log M)
```

### 适用场景

- 已排序数据
- 大数据量
- 外部排序

---

## 阶段 4：自适应 Join（v3.0）

### 策略选择

```rust
fn choose_join_strategy(
    left_stats: &Stats,
    right_stats: &Stats,
    memory_available: usize,
) -> JoinStrategy {
    let left_size = left_stats.rows * tuple_size;
    let right_size = right_stats.rows * tuple_size;
    
    if left_size + right_size < memory_available {
        // 内存足够，使用 Hash Join
        JoinStrategy::Hash
    } else if left_stats.sorted && right_stats.sorted {
        // 已排序，使用 Sort-Merge
        JoinStrategy::SortMerge
    } else if left_stats.rows < 1000 || right_stats.rows < 1000 {
        // 小表，使用 Nested Loop
        JoinStrategy::NestedLoop
    } else {
        // 大数据量，分区 Hash Join
        JoinStrategy::PartitionedHash
    }
}
```

### 运行时切换

```rust
struct AdaptiveJoin {
    strategy: JoinStrategy,
    stats: RuntimeStats,
}

impl AdaptiveJoin {
    fn maybe_switch(&mut self) {
        if self.stats.actual_rows > self.stats.estimated_rows * 5 {
            // 重新评估策略
            self.strategy = self.choose_new_strategy();
        }
    }
}
```

---

## 成本对比

| 算法 | 时间复杂度 | 空间复杂度 | 适用场景 |
|------|-----------|-----------|----------|
| Nested Loop | O(N×M) | O(1) | 小表 |
| Hash Join | O(N+M) | O(min(N,M)) | 等值连接 |
| Sort-Merge | O(NlogN+MlogM) | O(1) | 已排序 |

---

## 演进路线

```
v1.0 → Nested Loop Join
v2.0 → Hash Join
v2.x → Sort-Merge Join
v3.0 → 自适应 Join
```
