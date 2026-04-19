# v2.6.0 发布门禁检查清单（Phase B 重构版）

> **版本**: alpha/v2.6.0
> **当前阶段**: `Alpha`
> **更新日期**: 2026-04-19
> **验证状态**: ⏳ 部分待验证

---

## 一、门禁分类

| 分类 | 说明 | 影响 |
|------|------|------|
| A 类 | 必须通过 | 失败则阻塞发布/阶段晋级 |
| B 类 | 应通过 | 失败需风险审批与整改计划 |

### 证据要求（统一）

| 字段 | 说明 | 示例 |
|------|------|------|
| 执行命令 | 实际执行的命令 | `cargo test --test binary_format_test` |
| 执行日期 | 命令执行日期 | 2026-04-19 |
| Commit Hash | 代码版本 | `abc1234` |
| 结果摘要 | 关键结果 | "通过，0 failures" |
| 产物路径 | 报告或日志位置 | `artifacts/coverage/index.html` |

---

## 二、Alpha 门禁（当前阶段）

### A 类（必须）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 构建通过 | `cargo build --release` | 成功 | ✅ 已验证 | - |
| 格式检查 | `cargo fmt --check` | 无问题 | ✅ 已验证 | - |
| L0 冒烟 | `cargo test --test binary_format_test` | 通过 | ✅ 已验证 | - |
| L1 parser | `cargo test -p sqlrustgo-parser --lib` | 100% | ⏳ 待执行 | - |
| L1 planner | `cargo test -p sqlrustgo-planner --lib` | 100% | ⏳ 待执行 | - |
| L1 executor | `cargo test -p sqlrustgo-executor --lib` | 100% | ⏳ 待执行 | - |
| L1 storage | `cargo test -p sqlrustgo-storage --lib` | 100% | ⏳ 待执行 | - |
| L1 optimizer | `cargo test -p sqlrustgo-optimizer --lib` | 100% | ⏳ 待执行 | - |
| L1 transaction | `cargo test -p sqlrustgo-transaction --lib` | 100% | ⏳ 待执行 | - |
| L1 server | `cargo test -p sqlrustgo-server --lib` | 100% | ⏳ 待执行 | - |
| L1 vector | `cargo test -p sqlrustgo-vector --lib` | 100% | ⏳ 待执行 | - |
| L1 graph | `cargo test -p sqlrustgo-graph --lib` | 100% | ⏳ 待执行 | - |
| Clippy | `cargo clippy -- -D warnings` | 0 警告 | ⏳ 待验证 | - |

### B 类（应通过）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| SQL Corpus | `cargo test -p sqlrustgo-sql-corpus --lib` | ≥95% | ⏳ 待测 | - |
| 覆盖率趋势 | `cargo tarpaulin` | 较上版本提升 | ⏳ 待测 | `artifacts/coverage/` |

---

## 三、Beta 门禁

### A 类（必须）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| L2 CBO | `cargo test --test cbo_integration_test` | 100% | ⏳ 待执行 | - |
| L2 WAL | `cargo test --test wal_integration_test` | 100% | ⏳ 待执行 | - |
| L2 Regression | `cargo test --test regression_test` | 100% | ⏳ 待执行 | - |
| L2 E2E Query | `cargo test --test e2e_query_test` | 100% | ⏳ 待执行 | - |
| SQL Corpus | `cargo test -p sqlrustgo-sql-corpus --lib` | ≥95% | ⏳ 待测 | - |

### B 类（应通过）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 覆盖率 | `cargo tarpaulin` | ≥65% | ⏳ 待测 | `artifacts/coverage/` |
| TPC-H | `cargo bench --bench tpch_bench` | 通过 | ⚠️ 代码错误 | - |

---

## 四、RC 门禁

### A 类（必须）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 全量 L0~L2 | 见 TEST_PLAN.md 2.1-2.3 | 100% | ⏳ 待执行 | - |
| 覆盖率 | `cargo tarpaulin` | ≥70% | ⏳ 待测 | `artifacts/coverage/` |
| TPC-H SF1 | `cargo bench --bench tpch_bench` | 通过 | ⚠️ 代码错误 | `artifacts/benchmark/` |
| Sysbench | 外部工具 | ≥1000 QPS | ⏳ 待集成 | `artifacts/benchmark/` |
| 备份恢复 | 手动测试 | 通过 | ⏳ 待实现 | - |
| 崩溃恢复 | `kill -9` 测试 | 恢复 | ⏳ 待实现 | - |

### B 类（应通过）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 安全审计 | 安全分析报告 | 通过 | ⏳ 待审 | `docs/releases/v2.6.0/SECURITY_ANALYSIS.md` |
| 升级路径 | 从 v2.5.0 升级测试 | 通过 | ⏳ 待测 | - |

---

## 五、GA 门禁

### A 类（必须）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 72h 长稳 | 压力测试 | 稳定 | ⏳ 待实现 | `artifacts/benchmark/` |
| 全部 A 类为绿 | 汇总检查 | 100% | ⏳ 待汇总 | - |
| 发布文档完整 | 完整性检查 | 通过 | ⏳ 待检查 | `docs/releases/v2.6.0/` |
| 回滚演练 | 回滚测试 | 通过 | ⏳ 待演练 | - |

### B 类（应通过）

| 检查项 | 验证命令 | 阈值 | 状态 | 产物路径 |
|--------|----------|------|------|----------|
| 未解决缺陷 | 缺陷分级 | 有计划 | ⏳ 待整理 | Issue tracker |
| 维护计划 | v2.6.x 维护文档 | 明确 | ⏳ 待创建 | `docs/releases/v2.6.x/` |

---

## 六、测试覆盖清单（全部勾选）

| # | 测试类型 | 命令来源 | Alpha | Beta | RC | GA |
|---|----------|----------|-------|------|-----|-----|
| 1 | 单元测试 (L1) | TEST_PLAN.md 2.2 | ⏳ | ⏳ | ✅ | ✅ |
| 2 | 集成测试 (L2) | TEST_PLAN.md 2.3 | - | ⏳ | ✅ | ✅ |
| 3 | TPC-H Bench | INTEGRATION_TEST_PLAN.md 2.3 | - | - | ⚠️ | ⚠️ |
| 4 | Sysbench | 外部工具 | - | - | ⏳ | ✅ |
| 5 | 覆盖率 | tarpaulin | ⏳ | ⏳ | ✅ | ✅ |
| 6 | SQL Corpus | TEST_PLAN.md 4.1 | ⏳ | ✅ | ✅ | ✅ |
| 7 | 安装测试 | DEPLOYMENT_GUIDE.md | ⏳ | ⏳ | ✅ | ✅ |
| 8 | 升级测试 | MIGRATION_GUIDE.md | - | ⏳ | ⏳ | ✅ |
| 9 | 备份恢复 | 手动 | - | - | ⏳ | ✅ |
| 10 | 崩溃恢复 | 手动 | - | - | ⏳ | ✅ |
| 11 | 长稳测试 | 72h 压测 | - | - | - | ⏳ |

---

## 七、门禁统计模板

| 分类 | 总数 | ✅ 通过 | ⏳ 待执行 | ⚠️ 代码错误 | 🔴 失败 | 通过率 |
|------|------|---------|-----------|-------------|---------|--------|
| A 类 | 13 | 3 | 9 | 1 | 0 | 23% |
| B 类 | 2 | 0 | 2 | 0 | 0 | 0% |
| **总计** | **15** | **3** | **11** | **1** | **0** | **20%** |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
| 2.0 | 2026-04-19 | Phase B 重构：映射到真实 target，明确命令和状态 |

---

*门禁检查清单 v2.6.0*
*最后更新: 2026-04-19*
