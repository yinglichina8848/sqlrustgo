# v3.0.0 Alpha 集成状态

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06

---

## 开发完成度

| 模块/功能 | 状态 | 关键 PR |
|-----------|------|---------|
| 优化器桥接 (PredicatePushdown/ProjectionPruning/ConstantFolding) | ✅ 完成 | #358 |
| CBO 代价模型 | ⚠️ 部分 | SimpleCostModel 存在，未接入 planner |
| SQL Corpus | ✅ 100% | opencode |
| TPC-H SF=0.1 | ✅ 22/22 | — |
| INSERT...SELECT | ✅ | #368 |
| 窗口函数 (6个) | ✅ | window_executor.rs |
| CTE 执行 | ✅ | #363 |
| IN/EXISTS 子查询 | ✅ | cf4d733e |
| EXPLAIN ANALYZE | ✅ | #368 |
| INFORMATION_SCHEMA | ✅ | #357 |
| SHOW VARIABLES | ✅ | #364 |
| SSL/TLS | ✅ | rustls |
| 慢查询日志 | ✅ | — |
| 连接池 | ✅ | — |
| 查询缓存 | ✅ | — |
| Group Commit | ✅ | — |
| 事务隔离 (RC/SI/SSI) | ✅ | Proof-026 |

---

## A-Gate 门禁状态

| 门禁 | P0 必须 | 状态 |
|------|---------|------|
| A-OPT: 优化器激活 | 是 | ✅ 4/4 完成 |
| A-SQL: SQL 兼容性 | 是 | ✅ 6/6 完成 |
| A-EXEC: 执行引擎 | 是 | ✅ 基础完成 |
| A-TX: 事务隔离 | 是 | ✅ 基础完成 |
| A-HYG: 代码质量 | 是 | ⚠️ 部分待验证 |

---

## 待完成项（移至 Beta）

| 功能 | 原因 | 优先级 |
|------|------|--------|
| CBO 代价模型集成 | SimpleCostModel 未接入 planner | P0 |
| TPC-H SF=1 OOM 根治 | 内存治理未完成 | P0 |
| 索引选择 | 基于代价的索引选择 | P1 |
| Join 重排序 | 多表查询优化 | P1 |
| 覆盖率提升至 GA 目标 | optimizer/executor 缺口大 | P1 |

---

## 外部依赖

| 依赖 | 状态 | 说明 |
|------|------|------|
| Nomad 集群 | ✅ 健康 | 2 节点 |
| Gitea Actions Runner | ✅ 健康 | nomad-runner-v8 在线 |
| PostgreSQL | ✅ 健康 | Gitea DB |
| TPC-H 数据 | ⚠️ 部分 | SF=0.1 可用，SF=1 待生成 |