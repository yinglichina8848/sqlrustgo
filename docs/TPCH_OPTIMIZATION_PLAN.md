# TPC-H 性能优化计划

> 基于 2026-05-03 TPC-H 测试结果（SF0.01 / tpch-tiny, 866,602 rows）
> 当前通过率：20/22（Q3、Q10 超时）

---

## 一、已完成的错误修复（可追溯）

### 1.1 编译错误修复

| 日期 | 问题 | 修复 |
|------|------|------|
| 2026-05-03 | `RecordBatch` undeclared | 加 `use arrow::record_batch::RecordBatch;` |
| 2026-05-03 | `parse()` 返回 `Statement` 非 `Vec`，`for stmt in statements` 编译不过 | 改为 `if let Statement::CreateTable` |
| 2026-05-03 | `StringBuilder::finish()` → `GenericByteArray` 与 `Vec<ArrayRef>` 类型不匹配 | 改为 `Arc::new(builder.finish()) as ArrayRef` |
| 2026-05-03 | `tpch_import::run(args)` 缺少 `&` | 改为 `tpch_import::run(&args)` |

### 1.2 仓库配置更新

| 日期 | 变更 |
|------|------|
| 2026-05-03 | AGENTS.md: `develop/v2.8.0` → `develop/v2.9.0` |
| 2026-05-03 | AGENTS.md: 添加 Gitea remote `http://192.168.0.252:3000/openclaw/sqlrustgo.git` |
| 2026-05-03 | Git remote origin 已指向 Gitea（非 GitHub） |

### 1.3 回归测试

- `cargo test --all-features`: 86 passed, 0 failed（2026-05-03）

---

## 二、TPC-H 测试基线（2026-05-03）

### 数据规模
- SF0.01 / tpch-tiny
- 总行数：866,602（lineitem 600,572 / orders 150,000 / partsupp 80,000）
- 每次运行重新导入约 6s

### 测试结果

| Query | Status | Time(ms) | Rows |
|-------|--------|----------|------|
| Q1 | PASS | 3900.02 | 3 |
| Q2 | PASS | 944.62 | 20000 |
| Q3 | **TIMEOUT** | >60000 | - |
| Q4 | PASS | 1399.63 | 5 |
| Q5 | PASS | 953.30 | 1 |
| Q6 | PASS | 2967.59 | 1 |
| Q7 | PASS | 961.85 | 0 |
| Q8 | PASS | 978.85 | 0 |
| Q9 | PASS | 985.96 | 0 |
| Q10 | **TIMEOUT** | >60000 | - |
| Q11 | PASS | 949.59 | 0 |
| Q12 | PASS | 1109.95 | 1 |
| Q13 | PASS | 1053.95 | 15000 |
| Q14 | PASS | 968.97 | 0 |
| Q15 | PASS | 950.71 | 0 |
| Q16 | PASS | 1034.80 | 1 |
| Q17 | PASS | 956.48 | 0 |
| Q18 | PASS | 979.11 | 1 |
| Q19 | PASS | 975.19 | 0 |
| Q20 | PASS | 998.34 | 1000 |
| Q21 | PASS | 991.63 | 1 |
| Q22 | PASS | 996.92 | 0 |

**通过率：20/22（91%）| 超时：2/22**

---

## 三、性能瓶颈分析

### 3.1 瓶颈优先级排序（已纠正）

| 优先级 | 瓶颈 | 影响 |
|--------|------|------|
| P0 | GROUP BY 全量聚合（JOIN 后再聚合） | Q3/Q10 全排序 100k log 100k |
| P0 | ORDER BY 全排序（无 Top-K heap） | 所有带 ORDER BY 的查询 |
| P1 | JOIN 顺序不佳（无 filter selectivity） | 中间结果膨胀 |
| P1 | 0 行结果（Q7-Q9/Q11/Q14-Q15/Q17/Q19/Q22） | 正确性问题，非性能 |
| P2 | Nested Loop Join（非 Hash Join） | 多表 JOIN 查询 |
| P2 | Volcano pull-based（无向量化） | 整体吞吐 |

### 3.2 Q3/Q10 慢的根因

```
lineitem (600k) → JOIN orders → JOIN customer
→ GROUP BY (可能 100k groups)
→ SORT (100k log 100k)
→ LIMIT 10
```

问题：必须算完所有 group 才能知道 top 10，**不能 early stop**。

### 3.3 0 行结果可能原因

1. 日期比较是字符串比较（`"1995-3-5" > "1995-10-01"` 为 false）
2. EXISTS/IN 子查询未实现（Q21、Q22）
3. NULL 语义错误（`col = NULL` → UNKNOWN）
4. LIKE/字符串比较语义差异

---

## 四、优化计划

### 阶段 1：P0 — Top-K + GROUP BY Pre-Aggregation

#### Task 1.1: Top-K Heap 替代全排序

**目标**：所有带 `ORDER BY ... LIMIT K` 的查询，用 `BinaryHeap` 替代全排序

**修改文件**：`crates/executor/src/sort.rs`（或相关）

**实现**：
```rust
use std::collections::BinaryHeap;

pub struct TopKHeap {
    k: usize,
    heap: BinaryHeap<SortableRow>,
}

impl TopKHeap {
    pub fn push(&mut self, row: Row) {
        self.heap.push(SortableRow(row));
        if self.heap.len() > self.k {
            self.heap.pop();
        }
    }

    pub fn into_sorted(self) -> Vec<Row> {
        // heap → Vec → reverse
    }
}
```

**预期效果**：ORDER BY 从 O(N log N) → O(N log K)，K=10 时约 10x 提升

---

#### Task 1.2: GROUP BY Pre-Aggregation（Q3/Q10 专项）

**目标**：在 JOIN 之前先对 lineitem 做 GROUP BY，减少中间结果

**Q3 分析**：
```sql
SUM(l_extendedprice * (1 - l_discount))  -- 只依赖 lineitem
GROUP BY l_orderkey
```
→ 先在 lineitem 上按 `l_orderkey` 聚合（600k → ~150k 行）

**实现位置**：`crates/executor/src/group_by.rs`

**实现**：
```rust
pub struct PreAggregation {
    group_key: String,
    aggr_exprs: Vec<AggrExpr>,
}

// 在查询规划阶段识别可 pre-aggregate 的表达式
// 生成 pre-agg plan node，再接入 JOIN
```

---

### 阶段 2：P1 — JOIN 顺序优化 + 0行调试

#### Task 2.1: CBO 加入 Filter Selectivity

**目标**：CBO 估算成本时考虑 filter  selectivity，不只看 row_count

**修改文件**：`crates/optimizer/src/cost_based_optimizer.rs`

**实现**：
```rust
pub struct FilterStats {
    pub selectivity: f64,  // 过滤后保留比例
}

pub fn estimate_filter selectivity(expr: &Expr, stats: &TableStats) -> f64 {
    // 估算 LIKE/%/% 的 selectivity
    // 估算范围比较 < > 的 selectivity
}
```

---

#### Task 2.2: 0 行结果调试

**目标**：定位 Q7/Q8/Q9/Q11/Q14/Q15/Q17/Q19/Q22 0行原因

**方法**：在关键算子加 debug 打印
```rust
println!("DEBUG: scan lineitem filtered rows: {}", count);
println!("DEBUG: join lineitem-orders rows: {}", count);
println!("DEBUG: group_by groups: {}", count);
```

---

### 阶段 3：P2 — Hash Join + Vectorized（长期）

#### Task 3.1: Hash Join 替换 Nested Loop

#### Task 3.2: Vectorized Execution（Arrow Columnar + SIMD batch）

---

## 五、验收标准

| 阶段 | 目标 | 验证 |
|------|------|------|
| 阶段 1 完成 | Q3/Q10 < 60s | `tpch-bench --queries Q3,Q10` |
| 阶段 2 完成 | Q3/Q10 < 10s | 同上 |
| 阶段 2 完成 | 0行查询定位根因 | debug 输出 |
| 最终 | 22/22 PASS | `tpch-bench --queries all` |

---

## 六、当前分支状态

- 分支：`feature/tpch-binary-importer`
- Worktree：`~/sqlrustgo/.worktrees/tpch-import`
- PR：#207 → `develop/v2.9.0`
