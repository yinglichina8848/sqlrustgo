# 窗口函数实现设计文档

## Issue #955 - [v2.0][Phase2] 窗口函数实现

## 概述

实现 SQL 窗口函数支持，使 SQLRustGo 支持分析查询和排名操作。

## SQL 语法支持

```sql
-- 窗口函数语法示例
SELECT 
    name,
    department,
    salary,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank,
    RANK() OVER (ORDER BY salary DESC) as salary_rank,
    DENSE_RANK() OVER (PARTITION BY department ORDER BY salary DESC) as dept_rank,
    SUM(salary) OVER (PARTITION BY department) as dept_total,
    AVG(salary) OVER (PARTITION BY department ORDER BY salary DESC ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as dept_avg,
    LEAD(salary, 1) OVER (ORDER BY salary DESC) as next_salary,
    LAG(salary, 1) OVER (ORDER BY salary DESC) as prev_salary
FROM employees
ORDER BY department, salary DESC;
```

## 架构设计

### 1. Parser 层

```
Expression
  └── WindowFunction (新增)
        ├── func: WindowFunctionType (RowNumber, Rank, DenseRank, Lead, Lag, etc.)
        ├── args: Vec<Expression>
        ├── partition_by: Vec<Expression>
        ├── order_by: Vec<OrderBy>
        └── window_frame: WindowFrame
```

**新增 Token**:
- `ROW_NUMBER`, `RANK`, `DENSE_RANK`
- `LEAD`, `LAG`, `FIRST_VALUE`, `LAST_VALUE`, `NTH_VALUE`
- `OVER`, `PARTITION`, `ROWS`, `RANGE`
- `PRECEDING`, `FOLLOWING`, `CURRENT_ROW`, `UNBOUNDED`

### 2. Planner 层

**LogicalPlan**:
```rust
Window {
    input: Box<LogicalPlan>,
    window_expr: Vec<Expr>,
    partition_by: Vec<Expr>,
    order_by: Vec<OrderByExpr>,
    window_frame: WindowFrame,
}
```

**PhysicalPlan**:
```rust
WindowExec {
    input: Box<dyn PhysicalPlan>,
    window_function: WindowFunction,
    partition_by: Vec<usize>,  // partition column indices
    order_by: Vec<SortExpr>,   // order column specifications
    window_frame: WindowFrame,
}
```

### 3. Executor 层

**WindowExec 实现**:
- 按分区列分组
- 对每个分区内的数据按排序列排序
- 使用滑动窗口计算窗口函数
- 特殊处理:
  - `ROW_NUMBER`: 行号 (1, 2, 3, ...)
  - `RANK`: 排名 (1, 1, 3, ...) - 允许并列
  - `DENSE_RANK`: 密集排名 (1, 1, 2, ...)
  - `LEAD/LAG`: 前一行/后一行值
  - `SUM/AVG/COUNT`: 聚合窗口函数

## 实现阶段

### Phase 1: Parser 关键字和基础解析
- [x] 添加窗口函数 Token
- [ ] 实现 `parse_window_function()` 解析
- [ ] 实现 OVER/PARTITION BY/ORDER BY 解析

### Phase 2: Planner LogicalPlan
- [ ] 添加 Window LogicalPlan 变体
- [ ] 添加 Window 规划逻辑

### Phase 3: Planner PhysicalPlan  
- [ ] 添加 WindowExec
- [ ] 实现分区和排序逻辑

### Phase 4: Executor 实现
- [ ] 实现 ROW_NUMBER/RANK/DENSE_RANK
- [ ] 实现 LEAD/LAG
- [ ] 实现聚合窗口函数 (SUM/AVG/COUNT)

### Phase 5: 测试和优化
- [ ] 添加单元测试
- [ ] 添加集成测试
- [ ] 性能优化

## 测试用例

```sql
-- 测试 ROW_NUMBER
SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t;

-- 测试 RANK  
SELECT RANK() OVER (PARTITION BY dept ORDER BY salary) FROM t;

-- 测试窗口帧
SELECT SUM(salary) OVER (ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) FROM t;
```

## 依赖

- Issue #988 (Catalog) - 已完成 ✅
- Issue #956 (RBAC) - 后续依赖
- Issue #946 (向量化) - 最终聚合

---

*创建时间: 2026-03-28*
*AI: OpenCode B*
*状态: 开发中*