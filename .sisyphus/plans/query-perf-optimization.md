# SQLRustGo 查询性能优化：达到 SQLite 级别

## TL;DR

> **Quick Summary**: 通过三个核心改造（真实 Hash Join、迭代器扫描、谓词下推），将 TPC-H 查询性能从平均 619ms 提升至 ~100ms（SQLite 水平），整体提速约 **6-7x**。
> 
> **Deliverables**:
> - 真实 Hash Join 实现（HashMap 构建+探测，O(n+m)）
> - 迭代器扫描（避免全表克隆，零拷贝访问）
> - 谓词下推 + 投影裁剪（提前过滤，减少数据量）
> - 基于代价的 Join 重排序
> 
> **Estimated Effort**: Large（约 3-5 天）
> **Parallel Execution**: YES（4 waves，最大并行 6 任务）
> **Critical Path**: 扫描优化 → Hash Join → 谓词下推 → Join 重排序

---

## Context

### 现状（实测数据，SF=0.1）

| 查询 | SQLRustGo | SQLite | PostgreSQL | SQLRugGo/PG |
|------|-----------|--------|------------|-------------|
| Q1 | 1323ms | 280ms | 42ms | 31.5x |
| Q4 | 425ms | 4ms | 16ms | 26.9x |
| Q6 | 1003ms | 71ms | 29ms | 34.8x |
| Q13 | 256ms | 2ms | 10ms | 25.3x |
| Q14 | 230ms | 220ms | 30ms | 7.7x |
| Q19 | 231ms | 73ms | 26ms | 9.0x |
| Q20 | 231ms | 2ms | 12ms | 19.7x |
| Q22 | 233ms | 3ms | 11ms | 21.4x |
| **Avg** | **619ms** | **94ms** | **22ms** | **~28x** |

### 根因分析（代码层面）

通过深入代码审查发现三个核心瓶颈：

**1. "Hash Join" 是嵌套循环（O(n×m)）** — `local_executor.rs:967`
```rust
fn hash_inner_join(left, right, ...) {
    for lrow in left {      // O(n)
        for rrow in right { // O(m) — 没有 HashMap！
            // ... 对每一对行求值条件
        }
    }
}
```
- 150K orders × 600K lineitems = **900 亿次比较**
- 真正的 Hash Join 只需 O(n+m) = ~750K 次操作
- 这是 Q4(99x 慢)、Q13(122x 慢) 等 JOIN 查询性能差的直接原因

**2. scan() 全表克隆** — `engine.rs:517`
```rust
fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
    Ok(self.tables.get(table).cloned().unwrap_or_default())
    //                     ^^^^^^^ 每次都克隆整个表（60万行！）
}
```
- 每个查询都完全复制所有表数据
- 无索引辅助、无早期过滤、无列裁剪
- 对有 LIMIT 的查询也扫描全部数据

**3. 优化器全为空壳（TODO）** — `rules.rs`
- `PredicatePushdown.apply()` → `false` (TODO)
- `ProjectionPruning.apply()` → `false` (TODO)  
- `ConstantFolding.apply()` → `false` (TODO)
- 无代价模型、无统计信息、无 Join 重排序

---

## Work Objectives

### Core Objective
将 SQLRustGo 的 TPC-H 查询性能从平均 619ms 提升至 ~100ms（与 SQLite 同级），通过三个核心改造实现 6-7x 提速。

### Concrete Deliverables
1. 真正的 Hash Join 物理算子（HashMap 构建+探测）
2. 迭代器扫描（零拷贝访问、早期过滤、LIMIT 感知）
3. 谓词下推 + 投影裁剪优化规则（实际生效）
4. 基于代价的 Join 重排序
5. 性能基准测试脚本（与 SQLite/PostgreSQL 对比）

### Definition of Done
- [ ] TPC-H 8 个快速查询平均耗时 ≤ 150ms（目标 100ms）
- [ ] 所有已有 TPC-H 测试通过（32 个）
- [ ] clippy + test 全绿
- [ ] 对比报告：SQLRustGo vs SQLite 差距 ≤ 2x

### Must Have
- 真实 Hash Join（HashMap 构建/探测）
- 迭代器扫描（不克隆全表）
- 谓词下推规则（实际生效）
- 性能不退化于当前版本

### Must NOT Have (Guardrails)
- 不要改变外部 API（ScanExecutor trait 可扩展但需向后兼容）
- 不要引入 unsafe 代码
- 不要破坏已有 32 个 TPC-H 测试
- 不要修改 planner 的 AST 结构（仅扩展，不重写）
- 不要使用动态分发 (`dyn Any`) 做优化 — 用具体类型

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: cargo test
- **QA Policy**: 每个任务包含 agent-executed QA 场景

### 关键验证命令
```bash
# 功能正确性
cargo test --package sqlrustgo-bench --test tpch_test

# 性能基准
./scripts/run_tpch_bench.sh --sf 0.1 --queries Q1,Q4,Q6,Q13,Q14,Q19,Q20,Q22

# 对比测试
python3 scripts/compare_benchmarks.py /tmp/sqlrustgo-results.json /tmp/sqlite-results.json
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1（基础层 — 可全部并行，6 个任务）:
├── T1: 迭代器 ScanExecutor（zerocopy trait + impl）
├── T2: Scan 谓词过滤（filter during scan）
├── T3: 投影裁剪支持（column projection in scan）
├── T4: TableStats 统计收集（行数、NDV、min/max）
├── T5: 优化器规则框架重构（从 dyn Any → 具体类型）
└── T6: 常量折叠规则实现

Wave 2（核心层 — 依赖 Wave 1，4 个任务）:
├── T7: 真实 Hash Join 实现（HashMap 构建+探测）
├── T8: 谓词下推规则实现（依赖 T1, T2, T5）
├── T9: 投影裁剪规则实现（依赖 T1, T3, T5）
└── T10: 代价模型实现（依赖 T4）

Wave 3（高级层 — 依赖 Wave 2，3 个任务）:
├── T11: Join 重排序（基于 T10 代价模型）
├── T12: LIMIT 下推优化
└── T13: 小表广播优化（Broadcast Hash Join）

Wave 4（集成 — 依赖 Wave 3，2 个任务）:
├── T14: 端到端性能测试 + 回归测试
└── T15: SQLite/PostgreSQL 对比报告

Wave FINAL（审查 — 4 个并行审查任务）:
├── F1: Plan Compliance Audit (oracle)
├── F2: Code Quality Review (unspecified-high)
├── F3: Real Manual QA (unspecified-high)
└── F4: Scope Fidelity Check (deep)
```

**Critical Path**: T1 → T2 → T7 → T10 → T11 → T14 → F1-F4
**Max Parallel**: 6 (Wave 1)

---

## TODOs

- [ ] T1. **迭代器 ScanExecutor trait** — 新增零拷贝扫描接口

  **What to do**:
  - 在 `scan.rs` 中新增 `ScanIterator` trait（`fn next(&mut self) -> SqlResult<Option<&[Value]>>`）
  - 新增 `IterableScanExecutor` trait，方法 `scan_iter(table, columns)`
  - 为 `MemoryStorage` 实现 `IterableScanExecutor`：`BorrowedScan` 持有 `&HashMap<String, Vec<Record>>`
  - `BorrowedScan` 逐行返回 `&[Value]` 引用，不克隆数据
  - 保持原 `ScanExecutor` trait 不变（向后兼容）

  **Must NOT do**: 不修改 ScanExecutor trait 签名，不引入 Rc/RefCell

  **Recommended Agent Profile**:
  - **Category**: `deep` — 涉及 trait 设计、生命周期、存储层交互

  **Parallelization**: Wave 1（与 T2-T6 并行）| **Blocks**: T7, T8, T9 | **Blocked By**: None

  **QA Scenarios**:
  ```
  Scenario: 迭代器扫描 100 行返回正确数量
    Tool: Bash (cargo test)
    Steps: 1. MemoryStorage 插入 100 行 2. scan_iter() 遍历计数
    Expected: count == 100
    Evidence: .sisyphus/evidence/task-1-count.txt

  Scenario: 列裁剪返回指定列
    Tool: Bash (cargo test)
    Steps: 1. 插入 [1,"a",3.0] 2. scan_iter(columns=[0,2])
    Expected: row == [Value::Integer(1), Value::Real(3.0)]
    Evidence: .sisyphus/evidence/task-1-projection.txt
  ```

  **Commit**: `perf(executor): add IterableScanExecutor with zero-copy borrow scan`

---

- [ ] T2. **Scan 层谓词过滤** — 扫描时直接过滤行

  **What to do**:
  - 在 `BorrowedScan` 中加 `filter: Option<Predicate>` 字段
  - `next()` 中跳过不匹配的行（在返回引用前过滤）
  - 支持 eq/ne/lt/gt/like/in 基本谓词
  - 修改 `local_executor.rs` 的 `execute_scan` 传递 WHERE 条件

  **Must NOT do**: 不克隆行、不改变 Predicate 类型

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 1 | **Blocks**: T8 | **Blocked By**: T1

  **QA Scenarios**:
  ```
  Scenario: 过滤减少行数
    Tool: Bash (cargo test)
    Steps: 1. 插入 [1,"a"],[2,"b"],[3,"a"] 2. scan with filter col1="a"
    Expected: count == 2
    Evidence: .sisyphus/evidence/task-2-filter.txt

  Scenario: 无过滤返回全部
    Tool: Bash (cargo test)
    Steps: 1. scan 1000 行无 filter
    Expected: count == 1000
    Evidence: .sisyphus/evidence/task-2-nofilter.txt
  ```

  **Commit**: `perf(executor): add predicate filtering during scan iteration`

---

- [ ] T3. **投影裁剪实现** — 只扫描需要的列

  **What to do**:
  - 用 T1 的 `columns: Option<&[usize]>` 参数实现列裁剪
  - `BorrowedScan::next()` 按列索引提取子集
  - 实现 `ColumnSubset` 结构：包装 `&[Value]` + `&[usize]` 映射

  **Must NOT do**: 不影响无裁剪时的性能

  **Recommended Agent Profile**: **Category**: `quick`

  **Parallelization**: Wave 1 | **Blocks**: T9 | **Blocked By**: T1

  **QA Scenarios**:
  ```
  Scenario: 投影后只含指定列
    Tool: Bash (cargo test)
    Steps: 1. 插入 [1,"a",3.0] 2. columns=[0,2]
    Expected: row == [Value::Int(1), Value::Real(3.0)]
    Evidence: .sisyphus/evidence/task-3-prune.txt
  ```

  **Commit**: `perf(executor): add column projection pruning during scan`

---

- [ ] T4. **TableStats 统计收集** — 行数、基数、分布

  **What to do**:
  - 在 `stats.rs` 中实现 `TableStats` 结构：row_count, column_stats(HashMap<usize, ColumnStats>)
  - `ColumnStats`: ndv（distinct count）, null_count, min_val, max_val
  - 在 `MemoryStorage` 中维护统计（insert/delete 时增量更新）
  - 实现 `StatisticsProvider` trait：`fn table_stats(&self, table: &str) -> TableStats`

  **Must NOT do**: 不影响 insert 性能（O(1) 增量更新）

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 1 | **Blocks**: T10, T11 | **Blocked By**: None

  **QA Scenarios**:
  ```
  Scenario: 行数统计正确
    Tool: Bash (cargo test)
    Steps: 1. 插入 50 行 2. table_stats("t").row_count
    Expected: 50
    Evidence: .sisyphus/evidence/task-4-rowcount.txt

  Scenario: NDV 正确
    Tool: Bash (cargo test)
    Steps: 1. 插入 [1,"a"],[2,"a"],[3,"b"] 2. col_stats[1].ndv
    Expected: 2 (a,b)
    Evidence: .sisyphus/evidence/task-4-ndv.txt
  ```

  **Commit**: `feat(optimizer): add TableStats collection with incremental updates`

---

- [ ] T5. **优化器规则框架重构** — dyn Any → 具体类型

  **What to do**:
  - 修改 `rules.rs` 中 `Rule<Plan>` 的泛型参数为具体 `LogicalPlan` 类型
  - 定义 `LogicalPlan` 枚举（Scan, Filter, Project, Join, Aggregate, Sort, Limit）
  - 将 `RuleSet.apply(&mut dyn Any)` 改为 `RuleSet.apply(&mut LogicalPlan)`
  - 清理 `lib.rs` 中不再需要的 `NoOpOptimizer` 和 `dyn Any` 用法

  **Must NOT do**: 不修改 planner 的现有 plan 类型

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 1 | **Blocks**: T8, T9 | **Blocked By**: None

  **QA Scenarios**:
  ```
  Scenario: RuleSet 可以应用规则到 LogicalPlan
    Tool: Bash (cargo test)
    Steps: 1. 创建 RuleSet 2. 注册简单规则 3. apply 到 Scan 节点
    Expected: rule.apply 被调用，返回 true
    Evidence: .sisyphus/evidence/task-5-ruleset.txt
  ```

  **Commit**: `refactor(optimizer): rewrite rule framework with concrete LogicalPlan types`

---

- [ ] T6. **常量折叠规则实现** — 编译期求值

  **What to do**:
  - 在 `rules.rs` 实现 `ConstantFolding::apply(&LogicalPlan)`
  - 遍历表达式树：`1 + 2 * 3` → `7`, `'hello' || ' world'` → `'hello world'`
  - 处理 WHERE 子句中的常量表达式
  - 处理 SELECT 列表中的常量表达式

  **Must NOT do**: 不对运行时才能确定的值做折叠

  **Recommended Agent Profile**: **Category**: `quick`

  **Parallelization**: Wave 1 | **Blocks**: None | **Blocked By**: T5

  **QA Scenarios**:
  ```
  Scenario: 算术折叠
    Tool: Bash (cargo test)
    Steps: 1. WHERE 1+2 = 3 2. 应用 ConstantFolding
    Expected: 变为 WHERE true（全表返回）
    Evidence: .sisyphus/evidence/task-6-const.txt
  ```

  **Commit**: `feat(optimizer): implement constant folding rule`

---

- [ ] T7. **真实 Hash Join 实现** — HashMap 构建+探测 (O(n+m))

  **What to do**:
  - 在 `local_executor.rs` 中重写 `hash_inner_join()`：
    ```rust
    fn hash_inner_join(left, right, condition, ...) {
        // 1. 选小表做 build side
        let (build, probe) = if left.len() < right.len() { (left, right) } else { (right, left) };
        // 2. 构建 HashMap<JoinKey, Vec<&[Value]>>
        let mut hash: HashMap<Vec<Value>, Vec<&[Value]>> = HashMap::new();
        for row in build {
            let key = extract_join_key(row, condition, build_schema);
            hash.entry(key).or_default().push(row);
        }
        // 3. 探测
        for row in probe {
            let key = extract_join_key(row, condition, probe_schema);
            if let Some(matches) = hash.get(&key) {
                for m in matches {
                    let combined = merge_rows(row, m); // 不克隆！用引用
                    results.push(combined);
                }
            }
        }
    }
    ```
  - 实现 `extract_join_key()`：从 condition 提取等值 JOIN 列
  - 实现 `merge_rows()`：按 schema 合并左右行
  - 处理多列 JOIN 键（复合键）
  - 为 LEFT/FULL JOIN 保留未匹配行标记

  **Must NOT do**: 不回退到嵌套循环、不克隆全表

  **Recommended Agent Profile**: **Category**: `deep` — 核心算法优化

  **Parallelization**: Wave 2 | **Blocks**: T14 | **Blocked By**: T1, T2

  **QA Scenarios**:
  ```
  Scenario: 等值 JOIN 返回正确结果
    Tool: Bash (cargo test)
    Steps: 1. left=[(1,"a"),(2,"b")], right=[(1,"x"),(3,"y")]
           2. hash_inner_join on col0
    Expected: [(1,"a",1,"x")]
    Evidence: .sisyphus/evidence/task-7-join.txt

  Scenario: 大表 JOIN 性能 (150K × 600K)
    Tool: Bash (cargo test -- --ignored)
    Steps: 1. 运行 tpch_q13 (customer GROUP BY)
    Expected: < 50ms (当前 256ms)
    Evidence: .sisyphus/evidence/task-7-perf.txt
  ```

  **Commit**: `perf(executor): implement true hash join with HashMap build+probe (O(n+m))`

---

- [ ] T8. **谓词下推规则实现** — 将过滤条件下推到 scan 层

  **What to do**:
  - `PredicatePushdown::apply()` 遍历 LogicalPlan 树
  - 将 Filter 节点下推到 Scan 节点上方（或合并进 Scan）
  - 对于 Join 节点，将只涉及单表的条件下推到该表的 Scan 前
  - 使用 T2 的 `with_filter()` 接口

  **Must NOT do**: 不下推涉及多表的条件

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 2 | **Blocks**: T14 | **Blocked By**: T1, T2, T5

  **QA Scenarios**:
  ```
  Scenario: 过滤下推到扫描层
    Tool: Bash (cargo test)
    Steps: 1. LogicalPlan: Filter(col=1) → Scan(t1)
           2. apply PredicatePushdown
    Expected: Scan(t1, filter=col=1) — Filter 消解
    Evidence: .sisyphus/evidence/task-8-pushdown.txt
  ```

  **Commit**: `feat(optimizer): implement predicate pushdown to scan level`

---

- [ ] T9. **投影裁剪规则实现** — 裁剪无用列

  **What to do**:
  - `ProjectionPruning::apply()` 分析 SELECT 列表和 WHERE 条件
  - 确定每个表实际需要的列集合
  - 修改 Scan 节点的 `columns` 参数（裁剪不需要的列）
  - 处理子查询和聚合函数中的列引用

  **Must NOT do**: 不裁剪聚合/SORT 需要的列

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 2 | **Blocks**: T14 | **Blocked By**: T1, T3, T5

  **QA Scenarios**:
  ```
  Scenario: 裁剪无用列
    Tool: Bash (cargo test)
    Steps: 1. SELECT col0 FROM t(col0,col1,col2)
           2. apply ProjectionPruning
    Expected: Scan(t, columns=[0]) — 只扫描 col0
    Evidence: .sisyphus/evidence/task-9-projection.txt
  ```

  **Commit**: `feat(optimizer): implement column projection pruning`

---

- [ ] T10. **代价模型实现** — 基于统计的代价估算

  **What to do**:
  - `SimpleCostModel::estimate_cost(LogicalPlan)` 实现具体代价公式
  - Scan: `row_count * (1.0 - selectivity)`
  - Join: `left_rows * log(right_rows)` (hash join) 或 `left_rows * right_rows` (nested loop)
  - Filter: `input_rows * selectivity`
  - Sort: `input_rows * log(input_rows)`
  - 选择性估算：`=` → `1/ndv`, `<` → `0.33`, `BETWEEN` → `0.25`

  **Must NOT do**: 不过度复杂化（先实现基本公式）

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 2 | **Blocks**: T11 | **Blocked By**: T4

  **QA Scenarios**:
  ```
  Scenario: 代价合理排序
    Tool: Bash (cargo test)
    Steps: 1. Scan(100行) vs Scan(10000行) 2. 比较代价
    Expected: 100行代价 < 10000行代价
    Evidence: .sisyphus/evidence/task-10-cost.txt
  ```

  **Commit**: `feat(optimizer): implement statistics-based cost model`

---

- [ ] T11. **Join 重排序** — 代价驱动的表顺序优化

  **What to do**:
  - 修改 `join_planner.rs` 的 `build_join_plan()`：从贪心改为代价驱动
  - 用 T10 的代价模型评估不同 Join 顺序
  - 使用动态规划（DPccp 或 GOO 算法）找最优顺序（5-6 表内精确解）
  - 对 7+ 表使用启发式（最小表优先）

  **Must NOT do**: 不对 2 表 Join 做重排序（无收益）

  **Recommended Agent Profile**: **Category**: `deep`

  **Parallelization**: Wave 3 | **Blocks**: T14 | **Blocked By**: T7, T10

  **QA Scenarios**:
  ```
  Scenario: 小表优先 Join
    Tool: Bash (cargo test)
    Steps: 1. 3张表: 10行, 1000行, 100000行
           2. build_join_plan with cost model
    Expected: join order = [10, 1000, 100000]
    Evidence: .sisyphus/evidence/task-11-reorder.txt
  ```

  **Commit**: `perf(optimizer): implement cost-based join reordering`

---

- [ ] T12. **LIMIT 下推优化** — 提前终止扫描

  **What to do**:
  - 在 `BorrowedScan` 中支持 LIMIT：达到上限后立即停止迭代
  - 修改 `execute_limit` 传递 limit 值到 scan 层
  - 对 `SELECT * FROM large_table LIMIT 10` 类型查询极大加速

  **Must NOT do**: 不影响无 LIMIT 查询

  **Recommended Agent Profile**: **Category**: `quick`

  **Parallelization**: Wave 3 | **Blocks**: T14 | **Blocked By**: T1

  **QA Scenarios**:
  ```
  Scenario: LIMIT 提前终止
    Tool: Bash (cargo test)
    Steps: 1. 10000 行表 2. scan with LIMIT 5
    Expected: 只扫描 5 行（不是 10000）
    Evidence: .sisyphus/evidence/task-12-limit.txt
  ```

  **Commit**: `perf(executor): push LIMIT down to scan for early termination`

---

- [ ] T13. **小表广播优化** — Broadcast Hash Join

  **What to do**:
  - 在 `execute_hash_join()` 中：当 build side 行数 < 阈值（如 1000）时
  - 将整个 build side 作为闭包捕获的 HashMap（避免重复构建）
  - 对多个 probe side 复用同一个 hash 表（适用于多表 Join）

  **Must NOT do**: 不对大表做广播（内存风险）

  **Recommended Agent Profile**: **Category**: `quick`

  **Parallelization**: Wave 3 | **Blocks**: T14 | **Blocked By**: T7

  **QA Scenarios**:
  ```
  Scenario: 小表广播命中
    Tool: Bash (cargo test)
    Steps: 1. 50行 nations JOIN 600K lineitems
           2. 检查是否使用 broadcast 路径
    Expected: 使用 broadcast，build time > probe time
    Evidence: .sisyphus/evidence/task-13-broadcast.txt
  ```

  **Commit**: `perf(executor): add broadcast hash join for small tables`

---

- [ ] T14. **端到端性能测试 + 回归测试**

  **What to do**:
  - 运行 TPCH 8 个快速查询，记录性能
  - 对比优化前后：目标平均 ≤ 150ms（优化前 619ms）
  - 运行全部 32 个 TPC-H 测试确保无回归
  - 运行 `cargo clippy` + `cargo test --all-features`

  **Must NOT do**: 不跳过任何已有测试

  **Recommended Agent Profile**: **Category**: `unspecified-high`

  **Parallelization**: Wave 4 | **Blocks**: T15 | **Blocked By**: T7-T13

  **QA Scenarios**:
  ```
  Scenario: 性能达标
    Tool: Bash
    Steps: 1. cargo test tpch_test 2. 运行 bench 脚本
    Expected: 32 tests pass, avg < 150ms
    Evidence: .sisyphus/evidence/task-14-perf.txt
  ```

  **Commit**: `test: end-to-end performance validation and regression tests`

---

- [ ] T15. **SQLite/PostgreSQL 对比报告**

  **What to do**:
  - 用统一脚本运行三者的 TPC-H 查询
  - 生成对比 markdown 表格 + JSON 数据
  - 输出到 `benchmark_results/tpch-comparison-{date}.md`

  **Must NOT do**: 不修改数据库配置

  **Recommended Agent Profile**: **Category**: `writing`

  **Parallelization**: Wave 4 | **Blocks**: None | **Blocked By**: T14

  **QA Scenarios**:
  ```
  Scenario: 报告可读
    Tool: Bash
    Steps: 1. python3 scripts/compare_benchmarks.py
    Expected: 生成 md+json 文件，格式规范
    Evidence: .sisyphus/evidence/task-15-report.md
  ```

  **Commit**: `docs: add TPC-H comparison report with SQLite/PostgreSQL`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  逐条检查 Must Have / Must NOT Have，验证：true Hash Join 实现（非嵌套循环）、迭代器扫描存在、谓词下推生效、API 向后兼容、无 unsafe。
  Output: `Must Have [4/4] | Must NOT Have [4/4] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  `cargo clippy --all-features -- -D warnings` + `cargo test --all-features`。检查：unused imports、clone 减少量、HashMap 使用正确性。
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  运行 TPC-H 8 个快速查询，记录每个耗时。运行全部 32 个测试。检查性能对比。
  Output: `Queries [8/8 pass] | Avg Latency [N ms] | Regression [NONE/N found] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  对每个 task 检查 diff：预期改动 vs 实际改动，确保无范围外修改。检查 "Must NOT do" 合规。
  Output: `Tasks [15/15 compliant] | Contamination [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

| Task | Message | Key Files |
|------|---------|-----------|
| T1 | `perf(executor): add IterableScanExecutor with zero-copy borrow scan` | `scan.rs` |
| T2 | `perf(executor): add predicate filtering during scan iteration` | `scan.rs`, `local_executor.rs` |
| T3 | `perf(executor): add column projection pruning during scan` | `scan.rs` |
| T4 | `feat(optimizer): add TableStats collection with incremental updates` | `stats.rs`, `engine.rs` |
| T5 | `refactor(optimizer): rewrite rule framework with concrete LogicalPlan types` | `rules.rs`, `lib.rs` |
| T6 | `feat(optimizer): implement constant folding rule` | `rules.rs` |
| T7 | `perf(executor): implement true hash join with HashMap build+probe (O(n+m))` | `local_executor.rs` |
| T8 | `feat(optimizer): implement predicate pushdown to scan level` | `rules.rs` |
| T9 | `feat(optimizer): implement column projection pruning` | `rules.rs` |
| T10 | `feat(optimizer): implement statistics-based cost model` | `cost.rs`, `stats.rs` |
| T11 | `perf(optimizer): implement cost-based join reordering` | `join_planner.rs` |
| T12 | `perf(executor): push LIMIT down to scan for early termination` | `scan.rs`, `local_executor.rs` |
| T13 | `perf(executor): add broadcast hash join for small tables` | `local_executor.rs` |
| T14 | `test: end-to-end performance validation and regression tests` | `bench/` |
| T15 | `docs: add TPC-H comparison report with SQLite/PostgreSQL` | `benchmark_results/` |
| F1-F4 | `chore: final verification wave — plan compliance, code quality, QA, scope` | N/A |

---

## Success Criteria

### 性能目标

| 查询 | 优化前 | 目标 | 备注 |
|------|--------|------|------|
| Q1 | 1323ms | ≤ 300ms | 4.4x |
| Q4 | 425ms | ≤ 20ms | 21x (JOIN 密集) |
| Q6 | 1003ms | ≤ 100ms | 10x |
| Q13 | 256ms | ≤ 10ms | 25x (简单查询) |
| Q14 | 230ms | ≤ 200ms | 1.15x |
| Q19 | 231ms | ≤ 100ms | 2.3x |
| Q20 | 231ms | ≤ 10ms | 23x |
| Q22 | 233ms | ≤ 10ms | 23x |
| **Avg** | **619ms** | **≤ 100ms** | **6x** |

### 验证命令
```bash
# 正确性
cargo test --package sqlrustgo-bench --test tpch_test  # 32/32 pass

# 代码质量
cargo clippy --all-features -- -D warnings               # 0 warnings

# 性能基准
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1,Q4,Q6,Q13,Q14,Q19,Q20,Q22 \
  --iterations 3
```

### Final Checklist
- [ ] 所有 32 个 TPC-H 测试通过
- [ ] 8 个快速查询平均耗时 ≤ 150ms
- [ ] clippy 0 warnings
- [ ] Hash Join 实际使用 HashMap（非嵌套循环）
- [ ] scan() 不再克隆全表
- [ ] 谓词下推规则实际生效（不再返回 false）
- [ ] 对比报告已生成

