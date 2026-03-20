# [Epic-02] 可观测性增强

## 概述

打造"可解释数据库"核心能力，这是教学场景的核心亮点。

**资源占比**: 30%
**优先级**: P0

---

## 核心亮点

> 💡 EXPLAIN ANALYZE 是让学员"讲清数据库执行过程"的核心能力
> - 每个算子耗时可见
> - 行数统计准确
> - 执行计划树清晰

---

## Issues

### [OBS-01] EXPLAIN 执行计划

**优先级**: P0
**工作量**: 200 行

**描述**: 实现 EXPLAIN 命令，输出执行计划树

**Acceptance Criteria**:
- [ ] `EXPLAIN SELECT * FROM orders` 输出执行计划
- [ ] 计划树格式清晰（缩进、层级）
- [ ] 支持所有物理算子

**输出示例**:
```
HashJoin
├── SeqScan on orders
│   └── Filter: o_orderdate >= '1993-10-01'
└── SeqScan on customer
```

---

### [OBS-02] EXPLAIN ANALYZE

**优先级**: P0
**工作量**: 250 行

**描述**: 实现 EXPLAIN ANALYZE，输出每个算子实际耗时和行数

**Acceptance Criteria**:
- [ ] `EXPLAIN ANALYZE SELECT * FROM orders` 输出耗时
- [ ] 每个算子显示 `Actual time=Xms`
- [ ] 每个算子显示 `rows=Y`

**输出示例**:
```
HashJoin (cost=1234.56 rows=1000)
├── SeqScan on orders (cost=123.45 rows=5000)
│   └── Filter: o_orderdate >= '1993-10-01'
└── SeqScan on customer (cost=100.00 rows=1000)

Actual time=15.2ms, 1000 rows
```

---

### [OBS-03] 算子级 Profiling

**优先级**: P1
**工作量**: 150 行

**描述**: 为每个物理算子添加性能指标收集

**Acceptance Criteria**:
- [ ] 每个算子记录执行时间
- [ ] 每个算子记录处理行数
- [ ] 汇总指标可导出

---

### [OBS-04] 格式化输出

**优先级**: P0
**工作量**: 50 行

**描述**: 实现 EXPLAIN 格式化输出

**Acceptance Criteria**:
- [ ] 支持 Text 格式（默认）
- [ ] 缩进一致
- [ ] 算子名称对齐

---

## 实现步骤

1. **Parser 扩展**
   - 添加 `EXPLAIN` 和 `ANALYZE` token
   - 添加 `ExplainStatement` AST 节点

2. **Planner 扩展**
   - 添加 `Explain` LogicalPlan 节点
   - 创建 `explain.rs` 模块

3. **Executor 扩展**
   - 为每个算子添加计时
   - 实现 `OperatorMetrics` 收集

4. **输出格式化**
   - 实现树形格式化
   - 实现 ANALYZE 统计输出

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/parser/src/token.rs` | 添加 EXPLAIN, ANALYZE token |
| `crates/parser/src/parser.rs` | 添加 ExplainStatement 解析 |
| `crates/planner/src/logical_plan.rs` | 添加 Explain 逻辑计划 |
| `crates/planner/src/explain.rs` | **NEW** - EXPLAIN 格式化逻辑 |
| `crates/planner/src/physical_plan.rs` | 添加 ExplainExec 物理算子 |
| `crates/executor/src/executor.rs` | 添加算子计时 |

---

## 测试验证

```sql
-- 测试 EXPLAIN
EXPLAIN SELECT * FROM orders WHERE o_orderdate >= '1993-10-01';

-- 测试 EXPLAIN ANALYZE
EXPLAIN ANALYZE SELECT * FROM orders;
```

---

## 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 计时开销影响性能 | 低 | 仅 ANALYZE 模式收集 |
| 输出格式不一致 | 低 | 制定格式规范 |

---

**关联 Issue**: OBS-01, OBS-02, OBS-03, OBS-04
**总工作量**: ~650 行