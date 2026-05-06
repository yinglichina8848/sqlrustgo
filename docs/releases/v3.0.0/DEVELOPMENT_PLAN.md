# SQLRustGo v3.0.0 开发计划

> **版本**: v3.0.0
> **日期**: 2026-05-05（审计修订版）
> **目标**: 生产级性能核心 + 基础设施完善
> **前置版本**: v2.9.0 RC
> **预计周期**: 12 周（Development）+ 6 周（ABRG）
> **审计依据**: [AUDIT_AND_SUPPLEMENT.md](./AUDIT_AND_SUPPLEMENT.md)

---

## 一、版本定位

v3.0.0 是"**从能用到好用**"的分水岭版本。不同于早期计划（MySQL 5.6 兼容版），本版本聚焦两个核心：

1. **性能核心**：关闭与 MySQL 的 112 倍 QPS 差距，达到生产可用的性能基线
2. **基础设施**：补齐 CBO、连接池、查询缓存、INFORMATION_SCHEMA、SSL/TLS 等生产必需组件

高级功能（触发器、分区表、全文索引、Auto Tuning、GTID）延后至 **v3.1.0**。

### 量化目标

| 指标 | v2.9.0 | v3.0.0 及格线 | v3.0.0 目标线 | v3.1.0 卓越线 |
|------|---------|-------------|-------------|-------------|
| Point Select QPS | ~2,000 | ≥10,000 | ≥20,000 | ≥50,000 |
| UPDATE QPS | ~950 | ≥5,000 | ≥10,000 | - |
| DELETE QPS | ~206 | ≥2,000 | ≥5,000 | - |
| TPC-H SF=1 | 22/22 (Q17/Q18 慢) | 22/22 可运行 | 22/22 p99<2s | - |
| SQL Corpus 通过率 | 92.6% | ≥95% | ≥98% | - |
| MySQL 5.7 兼容评分 | 56.7/100 | ≥65/100 | ≥75/100 | ≥85/100 |
| 覆盖率 | 84.18% | ≥85% | ≥85% | - |

---

## 二、五阶段开发结构

```
Phase 0: Debt Sprint（2周）── 偿还设计债务
Phase 1: Performance Pocket v1（4周）── 性能核心
Phase 2: SQL Completeness（3周）── SQL 补齐
Phase 3: Infrastructure（2周）── 基础设施
Phase 4: Architecture Hardening（1周）── 架构加固

Alpha → Beta → RC → GA（6周，ABRG 流程）
```

### 依赖关系

```
Phase 0 ──→ Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
             │              │
             └──→ F-05 ─────┘
             │              │
             └──→ F-11 ─────┘
             │
             └──→ PP-03 ──→ F-13（TLS 依赖连接池）
```

---

## 三、Phase 0: Debt Sprint（2周）

> **目标**: 按源码证据修复已知设计债务，确保后续开发在稳固基础上进行。

| # | 任务 | 源码证据 | 工时 | 验收标准 |
|---|------|---------|------|---------|
| D-01 | CBO 规则实现 | `optimizer/src/rules.rs:67,95,122` — 3 个 `// TODO: Implement` | 4d | 3 个 TODO 清零，规则返回 true（有实际变换） |
| D-02 | Planner CTE 断连修复 | Parser 有 `WithClause`，Planner `create_physical_plan_internal` 无 CTE 分支 | 2d | `LogicalPlan::With` 可被 planner 生成 |
| D-03 | 触发器 storage 层实现 | `file_storage.rs:1483` — `"Triggers not supported"` | 2d | CREATE TRIGGER 在 FileStorage 可用（基础场景） |
| D-04 | SSI 脆弱性加固 | `transaction/src/ssi.rs:287` — TLA+ cycle 注释 | 1d | 并发压力测试通过（100 并发, 无假阳性） |
| D-05 | 聚簇索引 ADR | B+Tree 无 clustered index 实现 | 0.5d | ADR 文档产出，明确实现路径和时间点 |

**Phase 0 验收**: `cargo clippy --all-features -- -D warnings` 通过；CBO 3 个 TODO 清零。

---

## 四、Phase 1: Performance Pocket v1（4周）

> **目标**: 一组互相依赖、必须同时交付的性能优化。任何一项缺失都会使其他优化收益被抵消。

### 关键路径: CBO → Query Cache → Connection Pool → Group Commit

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| PP-01 | CBO 完善（Index Selection + Join Reordering + 代价模型） | `optimizer/src/` | 7d | TPC-H Q1 执行时间减少 ≥50% |
| PP-02 | 连接池（Connection Pool, `max_connections`/`connection_pool_size` 可配置） | `mysql-server/src/` 或 `server/src/` | 5d | 100 并发 point_select 无连接失败 |
| PP-03 | 查询缓存 DML 自动失效 + LRU 可配置容量 | `executor/src/query_cache.rs` | 3d | INSERT 后同条件 SELECT 返回新数据 |
| PP-04 | Group Commit（WAL 批量 fsync, `group_commit_batch_size`/`group_commit_timeout_ms` 可配置） | `transaction/src/` 或 `storage/src/wal.rs` | 3d | 批量写入 QPS 提升 ≥2x |
| PP-05 | 批量 Insert 优化 | `executor/src/` | 2d | 1000 行批量插入 <100ms |

### 性能验证（Phase 1 完成后）

| 测试 | 基线 (v2.9.0) | 目标 (v3.0.0) |
|------|-------------|-------------|
| Point Select QPS | ~2,000 | ≥8,000（阶段目标，Phase 2 后继续优化至 10K+） |
| TPC-H SF=0.1 Q1 | 基线待测 | 减少 ≥50% |
| 写入吞吐 | 基线待测 | 提升 ≥2x |

---

## 五、Phase 2: SQL Completeness（3周）

> **目标**: 补齐 P0 SQL 功能缺口，达到 MySQL 5.7 兼容评分 ≥65/100。

| # | 任务 | 依赖 | 工时 | 验收标准 |
|---|------|------|------|---------|
| F-01 | INSERT...SELECT | Phase 1 PP-01（CBO） | 3d | 基础场景通过，数据正确 |
| F-02 | 窗口函数补全（NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE） | 无 | 4d | 6 个函数全部通过 SQL Corpus |
| F-03 | CTE 执行（WITH + WITH RECURSIVE） | Phase 0 D-02 | 4d | C-02 测试组全部通过（32 用例） |
| F-04 | SERIALIZABLE 隔离级别 + Gap Locking | Phase 0 D-04 | 4d | 幻读测试通过（并发 INSERT + SELECT 无幻读） |
| F-05 | 教学模式增强（`SQLRUSTGO_TEACHING_MODE=1` 适配新功能） | 无 | 1d | CBO/窗口函数/CTE 在教学模式下可禁用/可见化 |

---

## 六、Phase 3: Infrastructure（2周）

> **目标**: 补齐运维和监控基础设施。

| # | 任务 | 依赖 | 工时 | 验收标准 |
|---|------|------|------|---------|
| I-01 | INFORMATION_SCHEMA（TABLES/COLUMNS/STATISTICS/FILES） | 无 | 3d | `SHOW TABLES`/`SHOW COLUMNS` 等效查询通过 |
| I-02 | EXPLAIN ANALYZE（完整执行计划输出 + JSON/tree 格式） | Phase 1 PP-01 | 3d | TPC-H Q1 EXPLAIN 输出包含代价估算和行数预测 |
| I-03 | SSL/TLS 支持 | Phase 1 PP-02（连接池） | 2d | MySQL 客户端 `--ssl-mode=REQUIRED` 握手成功 |
| I-04 | 慢查询日志（`long_query_time` 阈值, 文件/syslog 输出） | 无 | 1d | 超阈值查询记录到日志文件 |
| I-05 | CI Gate 完善 | 无 | 1d | TPC-H/Sysbench/覆盖率趋势/MySQL 兼容测试进入 CI |

### CI Gate 新建清单

```
[NEW] tpch-gate          — TPC-H SF=0.1 全量，回归检测
[NEW] sysbench-gate      — Point/UPDATE/INSERT 各场景 QPS 对比 baseline
[NEW] coverage-trend      — 覆盖率趋势存储 + 下降告警（连续 3 次下降触发）
[NEW] mysql-protocol-test — mysql:5.7 容器握手测试
[NEW] chaos-gate-gitea   — 混沌工程 5 场景迁移至 Gitea CI
```

---

## 七、Phase 4: Architecture Hardening（1周）

> **目标**: 对齐 ARCHITECTURE_EVOLUTION.md 的 3.0 系统闭环目标。

| # | 任务 | 验收标准 |
|---|------|---------|
| A-01 | 模块边界审计 | 画出 v3.0.0 模块依赖图，确认无环依赖；删除 ≥10% 不必要公开接口（≥30 个 pub → pub(crate)） |
| A-02 | API 版本化 | 所有对外 API 标注 `#[deprecated]` 或 `#[since = "3.0.0"]` |
| A-03 | 升级兼容性验证 | 产出 v2.9.0 → v3.0.0 迁移指南（配置变更、SQL 行为变更、存储格式兼容性） |
| A-04 | 教学模式保持 | `SQLRUSTGO_TEACHING_MODE=1` 下运行教学 Lab 12 个实验，全部通过 |

### 3.0 系统闭环判断标准（来自 ARCHITECTURE_EVOLUTION.md）

```
✅ 是否可以替换 storage 而不改 executor？
✅ 是否可以替换 planner 而不改 API？
✅ AI 是否可拔掉而系统仍可运行？
```

Phase 4 结束时，以上三项必须全部为 ✅。

---

## 八、时间线

```
Week 1-2   (5/5-5/18):   Phase 0 — Debt Sprint
Week 3-6   (5/19-6/15):  Phase 1 — Performance Pocket v1
Week 7-9   (6/16-7/6):   Phase 2 — SQL Completeness
Week 10-11 (7/7-7/20):   Phase 3 — Infrastructure
Week 12    (7/21-7/27):  Phase 4 — Architecture Hardening
                         ↓ Development 阶段截止 ↓
Week 13-14 (7/28-8/10):  Alpha — 功能验证 + SQL Corpus ≥95%
Week 15-16 (8/11-8/24):  Beta — 全量测试 + TPC-H SF=1 + Sysbench
Week 17-18 (8/25-9/7):   RC — 所有 CI Gate 通过 + 文档完善
Week 19    (9/8-9/14):   GA — 最终审计 + 发布
```

---

## 九、ABRG 阶段细化规则

在原 ABRG 规则基础上增加例外条款：

| 阶段 | 允许 | 禁止 |
|------|------|------|
| **Alpha** | bug fix, test fix, documentation, CI 完善, **CBO 参数调优**（不改变算法, 仅调阈值） | 新功能, 新算法, 架构变更 |
| **Beta** | bug fix, test fix, documentation, **修正性能回归**（证明是 bug 而非设计变更） | 新功能, 新优化策略 |
| **RC** | 仅 crash/data-loss/corruption 修复, 文档完善 | 一切其他修改 |
| **GA** | 禁止修改 | 禁止修改 |

---

## 十、交付物检查清单

### Development 阶段完成

- [ ] CBO 3 个 TODO stub 清零
- [ ] Planner 支持 LogicalPlan::With
- [ ] 连接池实现 + 可配置
- [ ] 查询缓存 DML 自动失效
- [ ] Group Commit 实现
- [ ] INSERT...SELECT 实现
- [ ] 窗口函数补全（6 个新函数）
- [ ] CTE 执行（WITH + WITH RECURSIVE）
- [ ] SERIALIZABLE 隔离级别 + Gap Locking
- [ ] INFORMATION_SCHEMA（TABLES/COLUMNS/STATISTICS）
- [ ] EXPLAIN ANALYZE
- [ ] SSL/TLS 支持
- [ ] 慢查询日志
- [ ] CI Gate 5 项新建
- [ ] 模块边界审计 + 接口精简 ≥10%
- [ ] v2.9.0 → v3.0.0 迁移指南
- [ ] 教学模式 12 实验全部通过

### GA 验收

- [ ] Point Select ≥10,000 QPS（及格线）/ ≥20,000 QPS（目标线）
- [ ] UPDATE ≥5,000 QPS（及格线）
- [ ] DELETE ≥2,000 QPS（及格线）
- [ ] TPC-H SF=1 22/22 可运行无 OOM
- [ ] SQL Corpus ≥98%
- [ ] 覆盖率 ≥85%（optimizer ≥70%, planner ≥80%, storage ≥85%）
- [ ] 模块依赖图无环依赖
- [ ] 不必要公开接口删除 ≥10%
- [ ] 混沌工程 5 场景全部通过（Gitea CI）
- [ ] kill -9 崩溃恢复验证通过
- [ ] MySQL 5.7 容器兼容性测试通过

---

## 十一、运维增强（Operations Enhancement）

根据 v2.9.0 弱项分析（运维维度仅 35/100，差距 -65），以下运维/管理功能需整合入 v3.0.0：

### Added to Phase 3 (Infrastructure)

| 增量任务 | 工时 | 验收标准 | 对应弱项 |
|---------|------|---------|---------|
| I-06: SHOW VARIABLES / 系统变量体系 | 2d | `SHOW VARIABLES LIKE '%version%'` 返回正确 | §1.5 |
| I-07: 运维手册 | 2d | `docs/OPERATIONS_MANUAL.md` 备份/监控/配置指南 | §6.1 |
| I-08: 架构决策记录 ADR | 1d | `docs/ARCHITECTURE_DECISIONS.md` 正式 ADR 格式 | §6.1 |

### Added to Phase 4 (Architecture Hardening)

| 增量任务 | 工时 | 验收标准 | 对应弱项 |
|---------|------|---------|---------|
| A-05: 在线 DDL 基础 | 2d | `ALTER TABLE ADD COLUMN` 不阻塞读取 | §5.2 |
| A-06: mysqldump 功能完善 | 1d | `crates/tools/src/mysqldump.rs` 可导出完整 schema+data | §5.2 |
| A-07: 性能调优指南 | 1d | `docs/PERFORMANCE_TUNING.md` B+Tree/BufferPool/WAL 调优 | §6.1 |

### 延至 v3.1.0 的运维项

| 任务 | 原因 |
|------|------|
| performance_schema | 需要完整的 instrumentation 框架 |
| mysqladmin 等效工具 | CLI 工具，独立 crate |
| mysqlbinlog / WAL binlog 格式 | 需要 WAL 重构 |
| 在线添加索引 | 需要存储引擎支持并行索引构建 |
| 自动故障转移 | 需要分布式共识（Raft/Paxos） |
| 组复制 | 需要分布式共识 |
| AES-256 存储加密 | 需要安全 crate 改造 |
| 行级安全 (RLS) | 需要查询重写和权限系统增强 |

---

## 十二、P2 延后至 v3.1.0

以下任务从原计划 A 中延后，在 v3.1.0 实现：

| 任务 | 原因 |
|------|------|
| 事件调度器 (Event Scheduler) | 需先完成 INFORMATION_SCHEMA 基础设施 |
| 全文索引 (FULLTEXT) + 中文分词 | 独立子系统，工时长 |
| 空间数据 (GIS) | 低优先级，用户群小 |
| 分区表 (RANGE/KEY/HASH) | 需先完成存储引擎优化 |
| Auto Tuning | 依赖 CBO 和性能基线成熟 |
| GTID 复制 | 依赖分布式架构稳定 |
| 并行复制 (LOGICAL_CLOCK) | 需先完善 MTS 基础 |
| JSON 函数完整套件 | 基础 JSON 函数已存在（v2.9.0），完整套件延后 |

---

## 十二、风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| CBO 实现后 QPS 未达 10K 及格线 | 中 | 高 | 预留 2 周（Phase 1-2 之间）调优 buffer；分层验收（RC 仅需 10K） |
| SSI 修复引入新 bug | 中 | 高 | 先写并发压力测试（TDD），后修代码 |
| 4 Agent 并行导致合并冲突 | 高 | 中 | Phase 内按模块分配 Agent；Phase 间串行（0→1→2→3→4） |
| TPC-H SF=1 OOM 持续 | 中 | 中 | Phase 1 完成后先做内存 profiling 定位泄漏源 |
| 时间不足（12周 vs 预估 13-14周） | 高 | 高 | Phase 4 可压缩至半周（砍掉 ADR 以外的文档工作）；P2 已延后 |
| 教学模式功能退化 | 低 | 中 | Phase 4 A-04 专项检查；每个 F 任务需验证教学模式行为 |

---

## 十三、Agent 分组实施计划

| 组 | Agent | 主要任务 | 文档 |
|----|-------|---------|------|
| A 组 | opencode (openheart/sqlrustgo) | Phase 0-1 性能核心 + I-03 SSL/TLS | [A_GROUP_IMPLEMENTATION_PLAN.md](./A_GROUP_IMPLEMENTATION_PLAN.md) |
| B 组 | claude (yinglichina163/sqlrustgo) | Phase 0-2 SQL 功能 + TPC-H 扩展 + 形式化验证 | [B_GROUP_IMPLEMENTATION_PLAN.md](./B_GROUP_IMPLEMENTATION_PLAN.md) |
| C 组 | deepseek (yinglichina163/sqlrustgo) | Phase 3-4 基础设施 + 架构加固 | [C_GROUP_IMPLEMENTATION_PLAN.md](./C_GROUP_IMPLEMENTATION_PLAN.md) |

---

## 十四、Issue 跟踪清单

### 遗留 Issue（需在 v3.0.0 关闭）

| Issue | 标题 | 负责组 | 关闭条件 |
|-------|------|--------|---------|
| #234 | TPC-H 9/22 → 18/22 | B 组 | `tpch-bench --queries all` ≥18/22 通过，无 OOM |
| #235 | PROOF-026 Write Skew / SSI | B 组 | `formal_smoke.sh` 含 SSI + `ssi_integration_test` 通过 |
| #277 | TPC-H 三平台对比 | B 组 | `tpch_comparison.json` 含 4 平台数据 |
| #175 | TPC-H SF=0.1 测试 | B 组 | 22 查询 SF=0.1 全部运行并记录 |

### 新增 Issue（按 Phase）

| Issue | 标题 | 负责组 | Phase |
|-------|------|--------|-------|
| #263 | 优化器 CBO 准确性测试 | B 组 | Phase 0 |
| #353 | v3.0.0 开发总控 | Hermes | All |

---

## 十五、关联文档

| 文档 | 说明 |
|------|------|
| [A_GROUP_IMPLEMENTATION_PLAN.md](./A_GROUP_IMPLEMENTATION_PLAN.md) | A 组性能核心实施计划 |
| [B_GROUP_IMPLEMENTATION_PLAN.md](./B_GROUP_IMPLEMENTATION_PLAN.md) | B 组 SQL 功能 + TPC-H 实施计划 |
| [C_GROUP_IMPLEMENTATION_PLAN.md](./C_GROUP_IMPLEMENTATION_PLAN.md) | C 组基础设施实施计划 |
| [AUDIT_AND_SUPPLEMENT.md](./AUDIT_AND_SUPPLEMENT.md) | v3.0.0 计划审计与补充报告 |
| [../v2.9.0/WEAKNESS_ANALYSIS.md](../v2.9.0/WEAKNESS_ANALYSIS.md) | v2.9.0 弱项分析 |
| [../v2.9.0/V3_0_0_DEVELOPMENT_PLAN.md](../v2.9.0/V3_0_0_DEVELOPMENT_PLAN.md) | 计划 B（原 v2.9.0 视角的 v3.0.0 计划） |
| [../VERSION_ROADMAP.md](../VERSION_ROADMAP.md) | 版本路线图 |
| [../../ARCHITECTURE_EVOLUTION.md](../../ARCHITECTURE_EVOLUTION.md) | 架构演进推演（1.0 → 4.0） |

---

*计划版本: 3.0（分组整合版）*
*修订日期: 2026-05-06*
*修订依据: 添加 Agent 分组计划 + Issue 跟踪清单*
*前版: 2.0 (2026-05-05, 审计修订版)*
