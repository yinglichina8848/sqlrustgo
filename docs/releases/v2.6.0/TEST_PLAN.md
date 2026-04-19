# v2.6.0 测试计划（对齐 v2.7.0 综合口径）

> 版本: `v2.6.0`  
> 更新日期: 2026-04-19  
> 目标: 在当前 Alpha 阶段建立“可执行、可追溯、可发布”的测试与评估基线

---

## 一、目标

1. 全面覆盖：单元、集成、性能、覆盖率、SQL Corpus、TPC-H、Sysbench、安装、备份恢复、崩溃恢复。
2. 全链路验证：每项 P0/P1 功能必须具备可执行测试证据。
3. 发布可用：测试计划直接映射门禁清单。

---

## 二、测试分层

### L0 冒烟（5-10 分钟）

1. 构建与静态检查
2. 核心 SQL 冒烟
3. 最小安装与启动验证

### L1 模块回归（20-40 分钟）

1. parser/planner/optimizer/executor/storage/transaction/server 单测
2. vector/graph/gmp/unified 模块单测

### L2 集成回归（40-90 分钟）

1. SQL 全链路集成
2. WAL/事务/FK/恢复集成
3. SQL + Vector + Graph 混合路径

### L3 深度验证（夜间）

1. TPC-H
2. Sysbench
3. 崩溃恢复
4. 备份恢复

---

## 三、测试范围与要求

## 3.1 单元测试

要求：
1. 关键模块通过率 100%
2. 失败必须关联 issue 并标注 pre-existing 或新引入

## 3.2 集成测试

要求：
1. P0 场景通过率 100%
2. 全量场景通过率 >= 95%

## 3.3 覆盖率

目标：
1. 全仓覆盖率 >= 70%
2. 关键模块覆盖率：
- parser >= 80%
- executor >= 75%
- storage >= 75%
- transaction >= 80%

## 3.4 SQL Corpus

目标：
1. SQL Corpus 通过率 >= 95%
2. P0 语法（SELECT/INSERT/UPDATE/DELETE/JOIN/GROUP/HAVING）100%

## 3.5 TPC-H

目标：
1. SF1 全量正确性 100%
2. 关键查询性能不低于当前基线

## 3.6 Sysbench

目标：
1. `point_select`、`read_only`、`read_write` 三场景可运行
2. 输出 QPS + P99 + 错误率

## 3.7 安装与升级测试

目标：
1. 安装文档命令可执行
2. 从 v2.5.0 升级到 v2.6.0 不破坏核心功能

## 3.8 备份/恢复/崩溃测试

目标：
1. 全量备份恢复通过
2. `kill -9` 崩溃恢复通过
3. 数据一致性校验通过

---

## 四、综合评估输出

必须产出：
1. `COVERAGE_REPORT.md`
2. `BENCHMARK.md` 或 `PERFORMANCE_REPORT.md`
3. `SECURITY_ANALYSIS.md`（并补安全测试证据）
4. `TEST_SUMMARY.md`（建议新增）

每份报告必须包含：
1. 环境信息
2. 命令与参数
3. 结果摘要
4. 产物路径

---

## 五、执行节奏

1. 每日：L0 + L1
2. 每周：L2 全量
3. 夜间：L3（TPC-H/Sysbench/恢复）
4. 发布前：全量 L0~L3 + 报告冻结

---

## 六、与门禁映射

1. Alpha：L0/L1 全绿 + 恢复链路通过
2. Beta：L2 全绿 + SQL Corpus >= 95%
3. RC：覆盖率达标 + TPC-H + Sysbench
4. GA：72h 长稳 + 备份恢复 + 崩溃恢复 + 回滚演练

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
| 2.0 | 2026-04-19 | 对齐 v2.7.0 综合测试口径，扩展为全覆盖测试计划 |
