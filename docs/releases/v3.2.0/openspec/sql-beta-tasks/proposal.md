## Why

SQL-1 和 SQL-2 是 Beta Gate 的 P1 任务，需要完成以满足 Beta Gate 要求。

- SQL-1: RECURSIVE CTE 支持目前部分实现，递归查询执行存在 bug
- SQL-2: Performance Schema 已有基础结构，需要扩展到 ≥60% 覆盖率

## What Changes

### SQL-1: RECURSIVE CTE 修复
- 修复递归 CTE 执行中的 bug（evaluate_binary_op 算术运算）
- 添加全面的 CTE 测试（递归、非递归、多重 CTE）

### SQL-2: Performance Schema 扩展
- 添加 setup_actors, setup_instruments 表
- 添加 events_statements_current/history
- 添加 events_waits_current/history（ring buffer）
- 添加 events_statements_summary_by_digest
- 添加 global_events 聚合

## Capabilities

### SQL-1: RECURSIVE CTE
- `sql-recursive-cte`: 修复递归 CTE 解析和执行
- 支持递归查询的迭代执行
- 支持 UNION ALL 在递归 CTE 中

### SQL-2: Performance Schema
- `sql-performance-schema`: 扩展性能监控覆盖范围
- 添加 11+ 新表
- 目标覆盖率 ≥60%

## Impact

- 影响模块: parser, planner, executor, information-schema
- 测试: 添加 CTE 和 Performance Schema 测试用例
- 验收标准: Beta Gate SQL-1, SQL-2 检查通过
