# SQLRustGo v2.3 开发计划

> **版本**: 2.3
> **日期**: 2026-04-09
> **目标**: 单机并行执行 - 多核并行加速查询
> **前置条件**: v2.2 GA 发布
> **预计周期**: 2-3 周
> **Agent**: Subagent-Driven 多任务并行

---

## 1. 版本目标

v2.3 是"性能加速版"，实现多核并行加速查询，充分利用多核 CPU 资源。

### 核心指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 并行扫描加速比 | ≥ 3x (8核) | 1000万行扫描 |
| 并行 HashJoin 加速比 | ≥ 2.5x (8核) | 100万行 join |
| 并行 Aggregate 加速比 | ≥ 2x (8核) | COUNT/SUM 分组聚合 |

---

## 2. 现有基础设施分析

### 2.1 已有的并行执行组件

| 组件 | 位置 | 状态 | 说明 |
|------|------|------|------|
| RayonTaskScheduler | `executor/src/task_scheduler.rs` | ✅ 完整 | 基于 Rayon 的任务调度器 |
| ParallelVolcanoExecutor | `executor/src/parallel_executor.rs` | ✅ 完整 | 并行执行器封装 |
| ParallelSeqScan | `executor/src/parallel_executor.rs` | ✅ 完整 | 并行顺序扫描，分区处理 |
| ParallelHashJoin | `executor/src/parallel_executor.rs` | ✅ 完整 | 并行哈希连接，分区哈希 |
| TaskScheduler trait | `executor/src/task_scheduler.rs` | ✅ 完整 | 统一调度接口 |

### 2.2 现有实现分析

```rust
// RayonTaskScheduler - 基于 Rayon ThreadPool
pub struct RayonTaskScheduler {
    pool: Arc<ThreadPool>,
    active_tasks: Arc<AtomicUsize>,
}

// ParallelSeqScan - 数据分区并行处理
fn execute_parallel_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
    // 1. 计算分区边界
    let partitions = calculate_partitions(total_count, degree);
    // 2. 使用 rayon::IntoParallelIterator 并行处理
    let results = partitions.into_par_iter().map(...).collect();
}

// ParallelHashJoin - 分区哈希连接
fn execute_parallel_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
    // 1. 并行执行左右子节点
    let (left_rows, right_rows) = rayon::join(...);
    // 2. 分区哈希连接
    let results = self.partition_hash_join(...);
}
```

---

## 3. 任务分解

### 3.1 并行 Aggregate (P0)

| Issue | 任务 | 状态 | PR估算 |
|-------|------|------|--------|
| #1401 | 实现 ParallelAggregateExecutor | 待开发 | 5 |
| #1402 | 支持 COUNT/SUM/AVG 并行分组 | 待开发 | 3 |
| #1403 | 支持 GROUP BY 并行执行 | 待开发 | 4 |

**实现方案**:
```rust
// 并行聚合策略
fn execute_parallel_aggregate(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
    // 1. 并行扫描数据
    let data = self.execute_parallel_scan(child)?;
    
    // 2. 按组键分区
    let groups: HashMap<GroupKey, Vec<Value>> = data.rows
        .into_par_iter()
        .fold(HashMap::new, |mut acc, row| {
            let key = extract_group_key(&row);
            acc.entry(key).or_default().push(row);
            acc
        })
        .reduce(HashMap::new, |mut a, b| { a.extend(b); a });
    
    // 3. 并行计算聚合值
    let results: Vec<Vec<Value>> = groups
        .into_par_iter()
        .map(|(key, rows)| compute_aggregate(&key, &rows))
        .collect();
    
    Ok(ExecutorResult::new(results, 0))
}
```

### 3.2 并行 Filter 增强 (P1)

| Issue | 任务 | 状态 | PR估算 |
|-------|------|------|--------|
| #1411 | 实现 ParallelFilterExecutor | 待开发 | 3 |
| #1412 |谓词下推与并行化 | 待开发 | 2 |

### 3.3 性能基准测试 (P0)

| Issue | 任务 | 状态 | PR估算 |
|-------|------|------|--------|
| #1421 | 并行扫描基准测试 | 待开发 | 2 |
| #1422 | 并行 HashJoin 基准测试 | 待开发 | 2 |
| #1423 | 并行 Aggregate 基准测试 | 待开发 | 2 |

### 3.4 调度策略优化 (P1)

| Issue | 任务 | 状态 | PR估算 |
|-------|------|------|--------|
| #1431 | 自适应并行度选择 | 待开发 | 4 |
| #1432 | 数据局部性优化 | 待开发 | 3 |

---

## 4. 开发顺序

```
Week 1: 并行 Aggregate
  ├── #1401 ParallelAggregateExecutor
  └── #1402 COUNT/SUM/AVG 并行分组

Week 2: 基准测试 + 调度优化
  ├── #1421 并行扫描基准测试
  ├── #1422 并行 HashJoin 基准测试
  └── #1431 自适应并行度选择

Week 3: 完善与优化
  ├── #1423 并行 Aggregate 基准测试
  └── #1412 谓词下推优化
```

---

## 5. 验收标准

- [ ] ParallelAggregateExecutor 实现并通过测试
- [ ] 并行扫描在 8 核机器上达到 ≥ 3x 加速
- [ ] 并行 HashJoin 在 8 核机器上达到 ≥ 2.5x 加速
- [ ] 并行 Aggregate 在 8 核机器上达到 ≥ 2x 加速
- [ ] 基准测试报告生成

---

## 6. 关键依赖

- v2.2 CBO 优化器（用于决定并行度）
- Rayon 并行库（已引入）
- TaskScheduler trait（已存在）

---

## 7. 风险与备选方案

| 风险 | 可能性 | 影响 | 备选方案 |
|------|--------|------|----------|
| 并行聚合内存竞争 | 中 | 中 | 使用局部聚合减少shuffle |
| 负载不均衡 | 中 | 中 | 动态分区调整 |
| 小数据集并行开销 | 高 | 低 | 小数据集自动降级为顺序执行 |

---

## 8. 后续版本

- **v2.4**: 分布式执行框架（Exchange 算子）
- **v3.0**: 多节点并行查询

---

**创建 Issue 命令**:
```bash
gh issue create --title "[v2.3][P0] ParallelAggregateExecutor" --body "..." --label "enhancement"
gh issue create --title "[v2.3][P1] 并行 Filter 增强" --body "..." --label "enhancement"
gh issue create --title "[v2.3][P0] 基准测试" --body "..." --label "enhancement"
```
