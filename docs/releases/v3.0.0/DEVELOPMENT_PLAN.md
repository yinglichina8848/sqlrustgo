# SQLRustGo v3.0.0 开发计划（已同步实际状态）

> **版本**: v3.0.0
> **日期**: 2026-05-06（**源同步修订版**）
> **状态**: **Development 阶段已完成，已进入 Alpha**
> **实际开发周期**: 2026-05-05 ~ 2026-05-06（2 天实际开发，非计划 12 周）
> **注意**: 本文档已通过 git commit log 和代码审计核查，与实际分支 `develop/v3.0.0` 保持一致。

---

## 一、版本定位

v3.0.0 是"**从能用到好用**"的分水岭版本：

1. **性能核心**：关闭与 MySQL 的 112 倍 QPS 差距，达到生产可用的性能基线
2. **基础设施**：补齐 CBO 桥接、连接池、查询缓存、INFORMATION_SCHEMA、SSL/TLS 等生产必需组件

高级功能（触发器、分区表、全文索引、Auto Tuning、GTID）延后至 **v3.1.0**。

### 当前实测基准

| 指标 | v2.9.0 | **v3.0.0 实测** | GA 目标 |
|------|---------|----------------|---------|
| Point Select QPS | ~2,000 | **~7,312**（Regression from 24K, needs CBO） | ≥20,000 |
| UPDATE QPS | ~950 | **~42,427** | ≥10,000 |
| DELETE QPS | ~206 | **~62,352** | ≥5,000 |
| SQL Corpus 通过率 | 92.6% | **100%** | ≥98% |
| 覆盖率 | 84.18% | **待 Alpha 验证** | ≥85% |

**注意**: Point Select QPS 从 24K 降至 7K（优化器桥接引入的 overhead），需 CBO 代价模型完成后再优化。

---

## 二、已完成任务清单（100% 已合并到 develop/v3.0.0）

### D 系列 — 设计债务偿还（Phase 0）

| # | 任务 | Commit | 实际状态 | 验收 |
|---|------|--------|---------|------|
| D-01 | CBO 规则桥接 | `819fc55c`, `12fadaea` | ✅ 桥接到 crates/optimizer 真实实现 | 86 测试通过，1+2 折叠验证 |
| D-02 | Planner CTE 断连修复 | `e5de164c` | ✅ CTE inlining 实现 | LogicalPlan::With 可生成 |
| D-04 | SSI 脆弱性加固 | `6097ca77` | ✅ TLA+ 模型 + 并发测试 7/7 | 100 并发无假阳性 |

### PP 系列 — Performance Pocket v1（Phase 1）

| # | 任务 | Commit | 实际状态 | 验收 |
|---|------|--------|---------|------|
| PP-02 | 连接池 | `b589fecc` (opencode) | ✅ 可配置 `max_connections` / `connection_pool_size` |
| PP-03 | 查询缓存 | `628ca60a` (opencode) | ✅ DML 自动失效 + LRU 可配置 + 命中率统计 |
| PP-04 | Group Commit | `588df67b` (opencode) | ✅ WAL 批量 fsync |
| PP-06 | 内存治理 | `59974f48` | ✅ MemoryTracker + 512MB 限额 |

### F 系列 — SQL 功能补齐（Phase 2）

| # | 任务 | Commit | 实际状态 |
|---|------|--------|---------|
| F-01 | INSERT...SELECT | `3ab9c317` | ✅ 实现，不依赖 CBO |
| F-02 | 窗口函数 | `c5c37357` | ✅ NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE |
| F-03 | CTE 执行 | `c7bc91db` | ✅ WITH + CTE inlining |
| F-05 | 教学模式 | `63210314` | ✅ TeachingEndpoints 文档 |

### I 系列 — 基础设施（Phase 3）

| # | 任务 | Commit | 实际状态 |
|---|------|--------|---------|
| I-01 | INFORMATION_SCHEMA | `0fea7eb6` 等 | ✅ SHOW TABLES / SHOW COLUMNS / SHOW DATABASES / DESCRIBE |
| I-02 | EXPLAIN ANALYZE | `0fea7eb6` | ✅ 文本输出执行计划 + 实际行数 |
| I-03 | SSL/TLS | `crates/mysql-server/src/lib.rs` | ✅ rustls + 自签名证书 + make_tls_config() |
| I-04 | 慢查询日志 | `crates/query-stats/src/slow_query_log.rs` | ✅ SlowQueryLog 结构体实现 |
| I-05 | CI Gate | tpch-gate.yml, coverage-trend.yml | ✅ TPC-H + 覆盖率趋势 |
| I-06 | SHOW VARIABLES | `364` | ✅ 15 个 MySQL 兼容系统变量 |
| I-07 | 运维手册 | `875e7ad5` | ✅ docs/OPERATIONS_MANUAL.md |
| I-08 | ADR | `875e7ad5` | ✅ docs/ARCHITECTURE_DECISIONS.md (5 条) |

### A 系列 — 架构加固（Phase 4）

| # | 任务 | Commit | 实际状态 |
|---|------|--------|---------|
| A-02 | API 版本化 | `63210314` | ✅ `#[deprecated(since="3.0.0")]` + EngineConfig |
| A-03 | 升级兼容指南 | `63210314` | ✅ docs/MIGRATION_GUIDE_v3.md |
| A-04 | 教学模式 | `63210314` | ✅ docs/guides/TEACHING_MODE_GUIDE.md |
| A-05 | 在线 DDL | `63210314` | ✅ ALTER TABLE ADD/DROP/MODIFY/RENAME (copy-swap) |
| A-06 | mysqldump | `63210314` | ✅ 导出 CREATE TABLE + INSERT |
| A-07 | 性能调优指南 | `63210314` | ✅ docs/PERFORMANCE_TUNING.md |

### 形式化验证

| # | 任务 | Commit | 实际状态 |
|---|------|--------|---------|
| PROOF-026 | Write Skew / SSI | `6097ca77` | ✅ TLA+ 模型 + cfg + 7 个并发测试 |

### SQL Corpus

| 指标 | 实际值 | 验证方式 |
|------|--------|---------|
| 通过率 | **100%** (485/485) | `cargo test -p sqlrustgo-sql-corpus` |

---

## 三、部分完成 / 未完成的任务（需 Alpha 阶段继续）

| 任务 | 内容 | 实际状态 | 原因 |
|------|------|---------|------|
| PP-01 (核心) | **CBO 代价模型 + 索引选择** | ⚠️ 仅桥接完成 | SimpleCostModel 未接入 planner；无代价驱动的索引选择 |
| PP-01 (可选) | **Join 重排序** | ❌ 未实现 | 需要 cardinality estimation 基础 |
| PP-05 | **批量 Insert 优化** | ❌ 未实现 | 无相关代码 |
| F-04 | **SERIALIZABLE 隔离级别 + Gap Locking** | ❌ 未实现 | PROOF-026 仅覆盖 anomaly detection 场景 |
| I-05 (剩余) | **sysbench-gate, mysql-protocol-test, chaos-gate-gitea** | ❌ 未实现 | 仅 tpch-gate.yml + coverage-trend.yml 已创建 |
| A-01 | **模块边界审计（接口精简 ≥10%）** | ⚠️ 仅脚本 | scripts/arch/module_boundary_audit.sh 存在但未执行 |
| A-04 | **教学模式 12 实验验证** | ⚠️ 仅文档 | docs/guides/TEACHING_MODE_GUIDE.md 存在但未实证 |

---

## 四、Alpha 阶段剩余的测试/验证工作

### 基础验证 (Alpha-1)

- [ ] `cargo test --all-features --workspace` 全量通过
- [ ] 覆盖率 ≥50%（optimizer ≥40%, parser ≥50%, executor ≥45%）
- [ ] `cargo clippy --all-features` 零警告
- [ ] `cargo fmt --check` 零差异
- [ ] `check_docs_links.sh` 零死链

### 功能深度测试 (Alpha-2)

| 功能 | 测试点 |
|------|--------|
| INSERT...SELECT | 不同列数、类型转换、自增主键 |
| 窗口函数 6 个 | PARTITION BY + ORDER BY、NULL、边界帧 |
| CTE 执行 | 递归深度限制、多 CTE 引用 |
| SERIALIZABLE | PROOF-026 并发压力、幻读 |
| EXPLAIN ANALYZE | 代价估算对齐真实执行 |
| INFORMATION_SCHEMA | 对比真实 MySQL 行为 |

### 性能回归 (Alpha-3)

- [ ] TPC-H SF=1 22/22 无 OOM（p99 < 10s 初始）
- [ ] Sysbench 全场景（oltp_read_write 等）
- [ ] R9 性能回归脚本真正运行

### 混沌工程 (Alpha-4)

- [ ] kill -9 崩溃恢复测试
- [ ] 长稳测试 30min+

---

## 五、当前 Issue 跟踪清单

### Open Issues（Gitea）

| # | 标题 | 状态 |
|---|------|------|
| 353 | v3.0.0 开发总控 | META |
| 356 | v2.9.0 已冻结 | 公告 |
| 370 | Alpha 集成与测试 | 进行中 |
| 376 | T-01: Sysbench OLTP 适配 | 🔴 open |
| 377 | I-09: COM_MULTI 多语句 | 🔴 open |
| 378 | I-10: Prepared Statement 绑定 | 🔴 open |
| 379 | T-02: 事务状态机测试 | 🔴 open |
| 380 | F-06t: Optimizer 测试扩展 | 🔴 open |
| 381 | F-06p: Planner 测试扩展 | 🔴 open |
| 382 | M-02: TPC-H SF=1 CI Gate | 🔴 open |

### 已关闭的遗留 Issue

| # | 标题 | 关闭原因 |
|---|------|---------|
| 118 | SQL 兼容性 | SQL Corpus 96.7%（现已 100%） |
| 175 | TPC-H SF=0.1 | 随 #234 关闭 |
| 201 | Formal Verification Phase 2 | Phase B S0-S5 已闭环 |
| 234 | TPC-H 18/22 | 22/22 全部通过 |
| 235 | PROOF-026 Write Skew | 7/7 测试通过 |
| 277 | TPC-H 三平台对比 | SQLite 完成 |
| 342 | B 组实施计划 | 已实施完成 |

---

## 六、删除的内容（与原始计划不一致的项）

以下任务从原始计划中**删除**（因评估为不现实、不需要或已过期）：

| 任务 | 原始计划 | 删除原因 |
|------|---------|---------|
| Phase 0 D-03 | 触发器 storage 层实现 | 无实际需求，v3.1.0 再评估 |
| Phase 0 D-05 | 聚簇索引 ADR | 优先 PoC 再决议 |
| Phase 4 A-01 | 模块依赖图 + 接口精简 | 已创建审计脚本但未执行（重要性低） |
| Agent 三组并行 | A/B/C 组分配 | 实际开发为单线程串行，Agent 分配不适用 |
| 原 Issue #263 | 优化器 CBO 准确性测试 | Gitea 中无此 Issue，忽略 |
| 12 周时间线 | 2026-05-05 ~ 2026-08-31 | 实际 Dev 阶段 2 天完成 |
| A/B/C 组实施文档 | A/B/C_GROUP_IMPLEMENTATION_PLAN.md | 从未创建 |

---

## 七、实际版本状态总结

| 项目 | 值 |
|------|-----|
| **当前阶段** | Alpha（进入中） |
| **开发分支** | `develop/v3.0.0` |
| **最新 commit** | `7188fa1b` |
| **Cargo.toml 版本** | 2.5.0（未更新到 3.0.0） |
| **Phase 0-4 核心功能完成度** | ~85%（剩余 CBO 代价模型 + 少量收尾） |
| **Alpha 缺口文档** | ALPHA_GAPS.md |

---

*文档版本: 4.0（源同步版）*
*修订日期: 2026-05-06*
*修订依据: `git log origin/develop/v3.0.0` 审计 + Gitea API 核查*
*前版: 3.0 (2026-05-05, 分组整合版 — 已废弃)*
