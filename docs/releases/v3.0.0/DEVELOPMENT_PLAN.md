# SQLRustGo v3.0.0 开发计划（已同步实际状态）

> **版本**: v3.0.0
> **日期**: 2026-05-06（源同步版）
> **状态**: Development 阶段已完成，已进入 Alpha
> **实际开发周期**: 2026-05-05 ~ 2026-05-06（2 天，非计划 12 周）
> **当前分支**: `develop/v3.0.0` @ `10ffc0e3`

---

## 一、当前实测指标

| 指标 | 目标 | **当前** | 状态 |
|------|------|--------|------|
| Point SELECT QPS | ≥20,000 | **7,312**（待 CBO 优化） | 🟡 |
| UPDATE QPS | ≥10,000 | **42,427** | ✅ |
| DELETE QPS | ≥5,000 | **62,352** | ✅ |
| SQL Corpus | ≥98% | **100%** | ✅ |
| TPC-H SF=0.1 | 22/22 | **22/22** ~10.9s | 🟡 |

---

## 二、已完成任务（24 项全部合并）

| 类别 | 任务 | 状态 | 说明 |
|------|------|------|------|
| **优化器** | CBO 规则桥接 | ✅ | 3 规则真实调用，86 测试 |
| **缓存** | 查询缓存 LRU + DML 失效 | ✅ | opencode |
| **连接** | 连接池 Thread Pool | ✅ | opencode |
| **提交** | Group Commit WAL 批量 | ✅ | opencode |
| **INSERT** | INSERT...SELECT | ✅ | |
| **窗口函数** | NTILE/LEAD/LAG/等 6 函数 | ✅ | |
| **CTE** | WITH 子句执行 | ✅ | |
| **信息模式** | INFORMATION_SCHEMA | ✅ | SHOW TABLES/COLUMNS/DESCRIBE |
| **查询计划** | EXPLAIN ANALYZE | ✅ | |
| **传输安全** | SSL/TLS | ✅ | rustls + 自签名证书 |
| **慢查询** | 慢查询日志 | ✅ | |
| **CI 门禁** | CI Gate (TPC-H + coverage-trend) | ✅ | |
| **系统变量** | SHOW VARIABLES | ✅ | 15 变量 |
| **运维** | 运维手册 | ✅ | |
| **架构决策** | ADR 记录 | ✅ | 5 条 |
| **API** | API 版本化 + `#[deprecated]` | ✅ | |
| **迁移** | v2.9→v3.0 迁移指南 | ✅ | |
| **教学** | 教学模式 | ✅ | |
| **DDL** | 在线 DDL ADD/DROP/MODIFY/RENAME | ✅ | |
| **导出** | mysqldump 导出 | ✅ | |
| **调优** | 性能调优指南 | ✅ | |
| **内存** | PP-06 内存治理 (512MB 限额) | ✅ | |
| **形式化证明** | PROOF-026 Write Skew/SSI | ✅ | TLA+ 模型 + 7 测试 |
| **SQL 测试** | SQL Corpus 100% (485/485) | ✅ | |

---

## 三、未完成/剩余任务

| 优先级 | 任务 | 难度 | 负责人 | Issue |
|--------|------|------|--------|-------|
| **P0** | CBO 代价模型集成 (SimpleCostModel + 索引选择) | 🔴 5-7d | opencode | — |
| **P0** | Sysbench OLTP 完整适配 | 🟡 3d | opencode | #376 |
| **P0** | COM_MULTI 多语句执行 | 🟡 2d | opencode | #377 |
| **P0** | Prepared Statement 参数绑定修复 | 🟢 1d | opencode | #378 |
| **P0** | 事务状态机压力测试 | 🟡 2d | claude | #379 |
| **P1** | TPC-H SF=1 CI Gate | 🟡 1d | — | #382 |
| **P1** | Optimizer 测试扩展 | 🟢 2d | claude | #380 |
| **P1** | Planner 逻辑测试扩展 | 🟢 2d | claude | #381 |
| **P2** | 连接池并发压力测试 | 🟡 2d | — | — |
| **P2** | 覆盖率 ≥85% | 🟡 3d | — | — |

---

## 四、Agent 分工

| Agent | 工作目录 | 负责任务 |
|-------|---------|---------|
| **opencode** | `~/workspace/dev/openheart/sqlrustgo` | #376 #377 #378 + CBO 代价模型 |
| **claude** | `~/workspace/dev/yinglichina163/sqlrustgo` | #379 #380 #381 |
| **deepseek** | `~/workspace/dev/openheart/sqlrustgo` | 文档同步 + A-HYG 门禁 + 临时修复 |

---

## 五、当前分支

`develop/v3.0.0` @ `10ffc0e3`
Open PR: #387 (Phase 1 complete → main)
