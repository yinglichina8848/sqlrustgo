## 优先级: P0 | 预估: 5-7 天

## 目标

将现有的 `SimpleCostModel` 接入 planner，实现基于代价的索引选择、Join 重排序和物理计划优化。

**验收标准:**
- [] `EXPLAIN SELECT * FROM t WHERE id = 1` 选择索引扫描（非全表扫描）
- [] TPC-H Q1 执行时间减少 >=50%
- [] Point SELECT QPS >=18,000（过渡）/ >=20,000（最终）
- [] 所有 optimizer 测试通过（现有 86 + 新增 CBO 测试）
- [] Clippy 零警告、fmt 零差异

## 任务详情（6 阶段）

1. **CostModel Trait 重设计** — CostNode 接口 + PhysicalPlan 节点实现
2. **多索引选择** — 基于谓词选择性选择最优索引
3. **Join 重排序** — 左深/右深 + hash/nested_loop/sort_merge 代价比较
4. **EXPLAIN 代价输出** — 每个节点显示估计代价
5. **统计信息收集** — 列级统计/直方图/ANALYZE TABLE
6. **验证与调优** — 性能基准 + 回归测试

## 基础代码

- crates/optimizer/src/cost.rs — SimpleCostModel + CboOptimizer
- crates/planner/src/planner.rs — DefaultPlanner（已持有 cost_model）
- crates/planner/src/physical_plan.rs — 所有 PhysicalPlan 节点类型

## 工作分支

git checkout -b feat/opencode-v3-cbo develop/v3.0.0

详细任务文件: .agents/opencode-cbo-task.md
