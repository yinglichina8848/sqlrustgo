# 5 万行规模压力测试分析

> 版本：v1.0
> 日期：2026-02-18
> 测试规模：单表 50,000 行

---

## 一、测试场景假设

| 参数 | 值 |
|:-----|:---|
| 单表行数 | 50,000 |
| 列数 | 10 |
| 索引数 | 3 |
| Join 表数 | 2 |
| 每行大小 | ~100 bytes |

---

## 二、情况分析

### 2.1 情况 1：当前简单执行器（非向量化）

**问题**：
- 每行循环
- `Box<dyn>` 多态开销
- 频繁 clone

**估算**：
```
50,000 行过滤 = 50k 次 predicate eval
2 表 join = 50k × 50k = 2.5B（NestedLoop 会炸）
```

**结果**：
```
🚨 性能不可接受
```

### 2.2 情况 2：优化后 HashJoin

**内存计算**：
```
每行假设 100B
50k × 100B ≈ 5MB
完全可接受
```

**HashJoin 时间复杂度**：
```
O(n + m)
50k + 50k = 100k
毫无压力
```

### 2.3 情况 3：Aggregate

**计算**：
```
group by 1000 groups
hash map 1000 entries
内存开销小
```

---

## 三、关键瓶颈分析

| 模块 | 风险 | 说明 |
|:-----|:-----|:-----|
| Parser | 🟢 低 | 纯语法分析 |
| LogicalPlan | 🟡 中 | 结构复杂度 |
| Optimizer | 🔴 高 | 规则数量 |
| Join | 💀 极高 | NestedLoop 爆炸 |
| 内存管理 | 🔴 高 | 频繁分配 |

---

## 四、性能对比

### 4.1 NestedLoop Join

```
时间复杂度: O(n × m)
50k × 50k = 2,500,000,000 次比较

预估时间（每比较 100ns）:
2.5B × 100ns = 250 秒

🚨 完全不可接受
```

### 4.2 HashJoin

```
时间复杂度: O(n + m)
50k + 50k = 100,000 次操作

预估时间（每操作 100ns）:
100k × 100ns = 10ms

✅ 完全可接受
```

### 4.3 对比表

| Join 类型 | 时间复杂度 | 50k×50k 耗时 |
|:----------|:-----------|:-------------|
| NestedLoop | O(n × m) | ~250 秒 |
| HashJoin | O(n + m) | ~10 ms |
| SortMerge | O(n log n + m log m) | ~100 ms |

---

## 五、未来规模推演

| 行数 | 状态 | 需要优化 |
|:-----|:-----|:---------|
| 5万 | ✅ 安全 | HashJoin 即可 |
| 50万 | ⚠️ 需要 | 向量化执行 |
| 500万 | 🔴 需要 | 并行执行 |
| 5000万 | 💀 需要 | 分布式执行 |

---

## 六、优化策略

### 6.1 短期优化（5万行）

| 优化 | 效果 |
|:-----|:-----|
| HashJoin | 解决 Join 爆炸 |
| 索引扫描 | 减少扫描量 |
| 谓词下推 | 减少数据量 |

### 6.2 中期优化（50万行）

| 优化 | 效果 |
|:-----|:-----|
| 向量化执行 | 10x 性能提升 |
| 列式存储 | 减少内存访问 |
| 并行扫描 | 多核利用 |

### 6.3 长期优化（500万+）

| 优化 | 效果 |
|:-----|:-----|
| 分布式执行 | 水平扩展 |
| 分区表 | 减少单节点压力 |
| 物化视图 | 预计算 |

---

## 七、基准测试设计

### 7.1 测试用例

```rust
#[bench]
fn bench_seq_scan_50k(b: &mut Bencher) {
    let table = create_table(50_000);
    b.iter(|| {
        table.scan().collect::<Vec<_>>()
    });
}

#[bench]
fn bench_filter_50k(b: &mut Bencher) {
    let table = create_table(50_000);
    b.iter(|| {
        table.scan()
            .filter(|row| row.value > 100)
            .collect::<Vec<_>>()
    });
}

#[bench]
fn bench_hash_join_50k(b: &mut Bencher) {
    let left = create_table(50_000);
    let right = create_table(50_000);
    b.iter(|| {
        hash_join(&left, &right, "id")
    });
}

#[bench]
fn bench_aggregate_50k(b: &mut Bencher) {
    let table = create_table(50_000);
    b.iter(|| {
        table.scan()
            .group_by("category")
            .aggregate("value", AggFunc::Sum)
    });
}
```

### 7.2 目标性能

| 操作 | 目标时间 |
|:-----|:---------|
| SeqScan 50k | < 50ms |
| Filter 50k | < 100ms |
| HashJoin 50k×50k | < 500ms |
| Aggregate 50k | < 200ms |

---

## 八、内存分析

### 8.1 内存占用估算

| 组件 | 内存占用 |
|:-----|:---------|
| 表数据 | 5MB (50k × 100B) |
| HashJoin 哈希表 | 10MB |
| 中间结果 | 5MB |
| **总计** | ~20MB |

### 8.2 内存优化

| 优化 | 效果 |
|:-----|:-----|
| 流式处理 | 减少峰值内存 |
| 列式存储 | 减少内存占用 |
| 压缩 | 进一步减少 |

---

## 九、结论

### 9.1 当前状态

```
5万行规模：
├── SeqScan: ✅ 可接受
├── Filter: ✅ 可接受
├── NestedLoop Join: 🚨 不可接受
├── HashJoin: ✅ 可接受
└── Aggregate: ✅ 可接受
```

### 9.2 关键行动

```
优先级：
├── P0: 实现 HashJoin（解决 Join 爆炸）
├── P1: 实现谓词下推（减少数据量）
├── P2: 实现索引扫描（加速查询）
└── P3: 引入向量化（提升性能）
```

---

*本文档由 TRAE (GLM-5.0) 创建*
