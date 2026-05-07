# SQLRustGo 历史遗留问题追踪报告

> **版本**: 1.0
> **日期**: 2026-05-07
> **状态**: 🟡 进行中
> **目标**: 闭环追踪所有历史遗留问题，不允许忽略任何定义过但未落实的问题

---

## 一、报告目的

解决以下问题：
1. **定义多、落实少**: 许多功能/测试在历史版本中定义过，但未实现或未测试
2. **状态不透明**: 缺少情况说明和进度总结
3. **忽略问题**: 部分测试被标记为 `#[ignore]` 但未说明原因和完成计划
4. **门禁脱节**: 门禁检查与实际功能状态不匹配

---

## 二、历史遗留问题总表

### 2.1 v2.9.0 → v3.0.0 功能缺口（来自 WEAKNESS_ANALYSIS.md）

| 类别 | 功能 | MySQL | v2.9.0 | v3.0.0 | 缺口等级 | 当前状态 | 说明 |
|------|------|-------|--------|--------|----------|----------|------|
| 事件调度器 | CREATE EVENT | ✅ | ❌ | ❌ | P0 | ❌ 未实现 | |
| 全文索引 | FULLTEXT | ✅ | ❌ | ❌ | P0 | ❌ 未实现 | |
| GIS | GEOMETRY + ST_* | ✅ | ❌ | ❌ | P0 | ❌ 未实现 | |
| 窗口函数 | NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE | ✅ | ❌ | ✅ | - | ✅ 已实现 | F-06 |
| CTE | WITH 递归 | ✅ | ❌ | ✅ | - | ✅ 已实现 | F-07 |
| INSERT...SELECT | 完整 | ✅ | ❌ | ✅ | - | ✅ 已实现 | F-05 |
| 存储过程游标 | DECLARE CURSOR | ✅ | ⚠️ | ⚠️ | P1 | ⚠️ 部分实现 | 60% |
| 触发器 | BEFORE/AFTER | ✅ | ❌ | ⚠️ | P1 | ⚠️ 部分实现 | 30% |
| SERIALIZABLE | 完整 | ✅ | ❌ | ⚠️ | P1 | ⚠️ 部分实现 | 50% |
| Gap Locking | Next-Key Lock | ✅ | ❌ | ❌ | P2 | ❌ 未实现 | |
| JSON 函数 | 完整 | ✅ | ⚠️ | ⚠️ | P2 | ⚠️ 部分实现 | 50% |
| 行级安全 RLS | 完整 | ✅ | ❌ | ❌ | P2 | ❌ 未实现 | |
| CREATE SEQUENCE | 完整 | ✅ | ❌ | ❌ | P2 | ❌ 未实现 | |

### 2.2 存储引擎缺口

| 功能 | MySQL (InnoDB) | v3.0.0 | 状态 | 完成计划 |
|------|----------------|--------|------|----------|
| 聚簇索引 | 主键即数据 | ❌ | P2 | GA 门禁前 |
| 自适应哈希索引 | 热数据哈希加速 | ❌ | P2 | GA 门禁前 |
| Change Buffer | 辅助索引缓存 | ⚠️ | P2 | GA 门禁前 |
| 双写缓冲 | 防止部分写入 | ❌ | P2 | GA 门禁前 |
| 表压缩 | PAGE_COMPRESSED | ❌ | P2 | GA 门禁前 |

### 2.3 运维生态缺口

| 功能 | MySQL | v3.0.0 | 状态 | 完成计划 |
|------|-------|--------|------|----------|
| INFORMATION_SCHEMA | 完整 | ⚠️ | P1 | RC 门禁前 |
| performance_schema | 完整 | ❌ | P2 | GA 门禁前 |
| 慢查询日志 | 完整 | ✅ | - | ✅ 已实现 |
| mysqladmin | 完整 | ❌ | P2 | GA 门禁前 |
| mysqlbinlog | 完整 | ❌ | P2 | GA 门禁前 |
| 在线 DDL | INPLACE | ⚠️ | P1 | RC 门禁前 |

### 2.4 安全缺口

| 功能 | MySQL 5.7 | v3.0.0 | 状态 | 完成计划 |
|------|-----------|--------|------|----------|
| TLS 加密连接 | ✅ | ✅ | - | ✅ 已实现 |
| SSL/TLS | ✅ | ✅ | - | ✅ 已实现 |
| AES-256 存储加密 | ✅ | ❌ | P2 | GA 门禁前 |
| 行级安全 RLS | ✅ | ❌ | P2 | GA 门禁前 |
| 列级权限 | ✅ | ⚠️ | P2 | GA 门禁前 |
| 密码轮转 | ✅ | ❌ | P2 | GA 门禁前 |

---

## 三、测试文件追踪

### 3.1 稳定性测试（Beta Gate B-S1~B-S5）

| 测试文件 | 路径 | Gate ID | 定义测试数 | 实际测试数 | 通过数 | 忽略数 | 问题 | 行动项 |
|---------|------|---------|-----------|-----------|--------|--------|------|--------|
| concurrency_stress_test | `tests/concurrency_stress_test.rs` | B-S1 | 9 | 9 | 9 | 0 | 无 | ✅ 已就绪 |
| crash_recovery_test | `tests/crash_recovery_test.rs` | B-S2 | 9 | 8 | 8 | 0 | 文档说9实际8 | 🔧 文档已修正 |
| long_run_stability_test | `tests/long_run_stability_test.rs` | B-S3 | 10 | 10 | 0 | 10 | 全部 #[ignore] | 🔴 必须修复 |
| wal_integration_test | `tests/wal_integration_test.rs` | B-S4 | 未定 | 16 | 16 | 0 | 无 | ✅ 已就绪 |
| network_tcp_smoke_test | `tests/network_tcp_smoke_test.rs` | B-S5 | 8 | 6 | 6 | 0 | 文档说8实际6 | 🔧 文档已修正 |

### 3.2 压力测试（S-01~S-03）

| 测试文件 | 路径 | 定义 | 实际 | 状态 | 问题 | 行动项 |
|---------|------|------|------|------|------|--------|
| connection_pool_stress_test | `crates/server/tests/connection_pool_stress_test.rs` | S-01 | 11 | 11 | Bug-01 overflow 未复现 | ✅ 可能已修复 |
| query_cache_test | ❌ 不存在 | S-02 | 0 | 0 | ❌ 缺失 | 🔴 必须实现 |
| wal_crash_recovery_test | ❌ 不存在 | S-03 | 0 | 0 | ❌ 缺失 | 🔴 必须实现 |

### 3.3 CBO/Optimizer/Planner 测试

| 测试文件 | 路径 | Gate ID | 覆盖率目标 | 当前覆盖 | 状态 |
|---------|------|---------|-----------|---------|------|
| optimizer tests | `packages/sqlrustgo-optimizer/` | B12 | ≥70% | ? | 🔴 需验证 |
| planner tests | `tests/cbo_integration_test.rs` | B13 | ≥80% | ? | 🔴 需验证 |
| CBO Index Selection | `tests/cbo_integration_test.rs` | B10 | - | - | ✅ |
| CBO Join Cost | `tests/cbo_integration_test.rs` | B11 | - | - | ✅ |

---

## 四、门禁检查落实情况

### 4.1 Beta Gate (19 项)

| ID | 检查项 | 通过标准 | 脚本实现 | 测试文件 | 状态 | 问题 |
|----|--------|---------|---------|---------|------|------|
| B1 | Release Build | cargo build --release | ✅ | - | ⏳ | |
| B2 | 全量测试 ≥90% | cargo test --all-features | ✅ | - | ⏳ | |
| B3 | Clippy | clippy --all-features | ✅ | - | ⏳ | |
| B4 | Format | cargo fmt --check | ✅ | - | ⏳ | |
| B5 | 覆盖率 ≥75% | cargo llvm-cov | ✅ | - | ⏳ | |
| B6 | 安全扫描 | cargo audit | ✅ | - | ⏳ | |
| B7 | 文档链接 | check_docs_links.sh | ✅ | - | ⏳ | |
| B8 | TPC-H SF=0.1 22/22 | check_tpch.sh sf=0.1 | ✅ | - | ⏳ | |
| B9 | SQL Corpus ≥85% | cargo test -p sqlrustgo-sql-corpus | ✅ | - | ✅ 100% | |
| B10 | CBO Index Selection | test_should_use_index | ✅ | cbo_integration_test | ✅ | |
| B11 | CBO Join Cost | test_estimate_join_cost | ✅ | cbo_integration_test | ✅ | |
| B12 | CBO Optimizer Tests | cargo test -p sqlrustgo-optimizer | ✅ | - | ⏳ | 需验证覆盖率 |
| B13 | CBO Planner Tests | cargo test --test cbo_integration_test | ✅ | - | ⏳ | 需验证覆盖率 |
| B-S1 | concurrency_stress_test | 9/9 PASS | ✅ | concurrency_stress_test.rs | ✅ | |
| B-S2 | crash_recovery_test | 8/8 PASS | ✅ | crash_recovery_test.rs | ✅ | 文档已修正 |
| B-S3 | long_run_stability_test | 10/10 PASS (--ignored) | ⚠️ | long_run_stability_test.rs | 🔴 | 全部忽略，必须修复 |
| B-S4 | wal_integration_test | 零数据丢失 | ✅ | wal_integration_test.rs | ✅ | |
| B-S5 | network_tcp_smoke_test | 无连接泄漏 (6/6) | ✅ | network_tcp_smoke_test.rs | ✅ | 文档已修正 |

**Beta Gate 阻塞项**: B-S3 (long_run_stability_test)

### 4.2 RC Gate (12 项)

| ID | 检查项 | 通过标准 | 脚本 | 状态 |
|----|--------|---------|------|------|
| R1 | Release Build | cargo build --release | check_rc_v300.sh | ✅ |
| R2 | 全量测试 100% | cargo test --all-features | - | ⏳ |
| R3 | Clippy | clippy --all-features | - | ⏳ |
| R4 | Format | cargo fmt --check | - | ⏳ |
| R5 | 覆盖率 ≥85% | cargo llvm-cov | - | ⏳ |
| R6 | 安全扫描 | cargo audit | - | ⏳ |
| R7 | 文档完整性 | v3.0.0 docs exist | - | ⏳ |
| R8 | SQL Corpus ≥95% | cargo test -p sqlrustgo-sql-corpus | - | ⏳ |
| R9 | TPC-H SF=1 22/22 | check_tpch.sh sf=1 | - | ⏳ |
| R10 | Performance Baseline | check_regression.sh | - | 🔴 stub |
| R11 | Sysbench Gate | check_sysbench.sh | - | ⏳ |
| R12 | Formal Proof | check_proof.sh | - | ⏳ |

### 4.3 GA Gate (15 项)

| ID | 检查项 | 通过标准 | 脚本 | 状态 |
|----|--------|---------|------|------|
| GA-1 | Release Build | cargo build --release | check_ga_v300.sh | ✅ |
| GA-2 | 测试 100% | cargo test --all-features | - | ⏳ |
| GA-3 | Integration tests | run_integration.sh --quick | - | ⏳ |
| GA-4 | Clippy | clippy --all-features | - | ⏳ |
| GA-5 | Format | cargo fmt --check | - | ⏳ |
| GA-6 | 覆盖率 ≥85% | cargo llvm-cov | - | ⏳ |
| GA-7 | 安全扫描 | cargo audit | - | ⏳ |
| GA-8 | 文档链接 | check_docs_links.sh | - | ⏳ |
| GA-9 | TPC-H SF=1 22/22 | check_tpch.sh sf=1 | - | ⏳ |
| GA-10 | 性能回归 (5%) | check_regression.sh | - | 🔴 stub |
| GA-11 | Formal proofs ≥10 | docs/proof/*.json | - | ⏳ |
| GA-12 | Sysbench Gate | check_sysbench.sh | - | ⏳ |
| GA-13 | 文档完整性 | 8 份文档存在 | - | ⏳ |
| GA-14 | SQL Corpus ≥95% | cargo test -p sqlrustgo-sql-corpus | - | ⏳ |
| GA-15 | 版本一致性 | cargo version + docs | - | ⏳ |

---

## 五、必须修复的问题（按优先级）

### 🔴 P0 - 阻塞 Beta Gate

| # | 问题 | 当前状态 | 影响 | 修复方案 | 负责人 | 截止 |
|---|------|----------|------|----------|--------|------|
| 1 | long_run_stability_test 全部 #[ignore] | 10/10 忽略 | Beta Gate 失败 | 实现真实稳定性测试或移除 #[ignore] | 待分配 | Beta 前 |
| 2 | query_cache_test 不存在 | 缺失 | S-02 无法通过 | 创建测试文件并实现测试 | 待分配 | Beta 前 |
| 3 | wal_crash_recovery_test 不存在 | 缺失 | S-03 无法通过 | 创建测试文件并实现测试 | 待分配 | Beta 前 |

### 🟠 P1 - 阻塞 RC Gate

| # | 问题 | 当前状态 | 影响 | 修复方案 | 负责人 | 截止 |
|---|------|----------|------|----------|--------|------|
| 4 | TPC-H SF=1 OOM | 22/22 可运行但 OOM | RC Gate R9 | 增加内存或优化查询 | 待分配 | RC 前 |
| 5 | Performance Baseline stub | check_regression.sh 是 stub | RC/GA Gate 失败 | 实现性能回归检查 | 待分配 | RC 前 |
| 6 | optimizer 覆盖率 <70% | 待测量 | B12 无法通过 | 扩展测试 | 待分配 | RC 前 |
| 7 | planner 覆盖率 <80% | 待测量 | B13 无法通过 | 扩展测试 | 待分配 | RC 前 |

### 🟡 P2 - 建议 GA 前完成

| # | 问题 | 当前状态 | 说明 |
|---|------|----------|------|
| 8 | connection_pool_stress_test Bug-01 | ✅ 可能已修复 | 需验证 test_high_contention_stress |
| 9 | INFORMATION_SCHEMA 不完整 | 缺部分表 | P1 完成 |
| 10 | 在线 DDL 仅支持 COPY | 非 INPLACE | P1 完成 |
| 11 | SERIALIZABLE 仅 50% | 部分实现 | P1 完成 |
| 12 | 聚簇索引未实现 | P2 | GA 前 |
| 13 | performance_schema 未实现 | P2 | GA 前 |
| 14 | mysqladmin 未实现 | P2 | GA 前 |

---

## 六、跨版本任务（不阻塞当前版本发布）

以下任务可以在后续版本中完成，但必须有明确的计划和状态追踪：

| 任务 | 版本 | 说明 | 状态 | 计划 |
|------|------|------|------|------|
| 聚簇索引 | v3.1.0 | 主键即数据 | 未开始 | TBD |
| 自适应哈希索引 | v3.1.0 | 热数据哈希 | 未开始 | TBD |
| Change Buffer | v3.1.0 | 辅助索引缓存 | 部分实现 | TBD |
| performance_schema | v3.2.0 | 监控完善 | 未开始 | TBD |
| mysqladmin 等效 | v3.2.0 | 运维工具 | 未开始 | TBD |
| FULLTEXT 索引 | v3.2.0 | 全文搜索 | 未开始 | TBD |
| GIS 支持 | v3.3.0 | 地理空间 | 未开始 | TBD |
| CREATE EVENT | v3.3.0 | 事件调度 | 未开始 | TBD |

---

## 七、闭环追踪机制

### 7.1 问题状态定义

| 状态 | 定义 | 要求 |
|------|------|------|
| 🔴 未开始 | 完全没有实现 | 必须有完成计划 |
| 🔴 阻塞 | 有障碍阻止前进 | 必须有解决方案 |
| 🟠 进行中 | 正在开发 | 必须有进度报告 |
| 🟡 待验证 | 实现完成待测试 | 必须有测试验证 |
| ✅ 已完成 | 通过验证 | 必须有验证证据 |
| ⚠️ 故意跳过 | 有充分理由暂时不做 | 必须有文档说明原因和后续计划 |

### 7.2 报告更新频率

- **Beta Gate 前**: 每日更新
- **RC Gate 前**: 每周更新
- **GA Gate 前**: 每日更新
- **发布后**: 每版本更新

### 7.3 责任矩阵

| 角色 | 职责 |
|------|------|
| 开发者 | 实现功能，编写测试，更新状态 |
| 测试负责人 | 验证测试通过，维护门禁脚本 |
| 版本负责人 | 审核状态，决策跳过，更新报告 |

---

## 八、行动清单

### 立即行动（本周）

- [ ] **T-02**: 修复 long_run_stability_test 全部 #[ignore] 问题
- [ ] **S-02**: 创建 query_cache_test.rs
- [ ] **S-03**: 创建 wal_crash_recovery_test.rs
- [ ] **验证**: connection_pool_stress_test Bug-01 是否已修复

### 本月行动（Beta Gate 前）

- [ ] **T-03**: optimizer 测试覆盖率 ≥70%
- [ ] **T-04**: planner 测试覆盖率 ≥80%
- [ ] **M-02**: TPC-H SF=1 可运行无 OOM

### RC Gate 前

- [ ] **D-01**: CBO 代价模型集成
- [ ] **R10**: 实现 check_regression.sh
- [ ] 覆盖率 ≥85%

---

## 九、相关文档索引

| 文档 | 位置 | 用途 |
|------|------|------|
| 本报告 | docs/releases/v3.0.0/LEGACY_TRACKING_REPORT.md | 历史问题追踪 |
| Alpha 缺口 | docs/releases/v3.0.0/ALPHA_GAPS.md | Alpha 阶段整改清单 |
| Beta 门禁 | docs/releases/v3.0.0/BETA_GATE_MASTER_CONTROL.md | Beta 任务总控 |
| 历史遗留开发计划 | docs/releases/v3.0.0/HISTORICAL_LEGACY_DEVELOPMENT_PLAN.md | 功能开发追踪 |
| Beta Gate 脚本 | scripts/gate/check_beta_v300.sh | Beta 门禁检查 |
| RC Gate 脚本 | scripts/gate/check_rc_v300.sh | RC 门禁检查 |
| GA Gate 脚本 | scripts/gate/check_ga_v300.sh | GA 门禁检查 |

---

*本文档由 Claude 生成并维护。*
*每次门禁检查后更新状态。*
*禁止将任何问题标记为"已忽略"而不提供原因和后续计划。*

---

## 十、Issue 创建状态

### 10.1 Gitea Issue 创建结果

✅ **已成功创建所有 Issue**

| Issue # | 标题 | 优先级 | 门禁 | URL |
|---------|------|--------|------|------|
| #416 | query_cache_test.rs 创建 | P0 | S-02 | [#416](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/416) |
| #417 | wal_crash_recovery_test.rs 创建 | P0 | S-03 | [#417](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/417) |
| #418 | check_perf_baseline.sh 实现 | P0 | R10, GA-10 | [#418](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/418) |
| #419 | long_run_stability_test #[ignore] 修复 | P0 | B-S3 | [#419](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/419) |
| #420 | [总追踪] 历史遗留问题闭环追踪 | P0 | 全部 | [#420](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/420) |
