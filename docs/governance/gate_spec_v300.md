# v3.0.0 门禁规范 (Gate Specification)

> **版本**: 1.0
> **更新日期**: 2026-05-06
> **维护人**: hermes-z6g4
> **适用版本**: v3.0.0
> **前置版本**: v2.9.0 RC

> **SSOT 声明**: `gate_spec_v300.md` 是 v3.0.0 门禁定义的唯一权威来源。`DEVELOPMENT_PLAN.md`、`RELEASE_LIFECYCLE.md` 等文档中的门禁描述仅作引用，不得独立定义门禁检查项。

---

## 一、门禁概述

v3.0.0 采用四级门禁模型（Alpha → Beta → RC → GA），在 v2.9.0 基础上强化了性能门禁：

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

| 门禁 | 名称 | 目标 | 覆盖率目标 | 性能目标 |
|------|------|------|-----------|---------|
| A-Gate | Alpha Gate | Phase 开发完成 | ≥50% | 基线建立 |
| B-Gate | Beta Gate | 功能冻结 | ≥75% | TPC-H SF=0.1 22/22 |
| R-Gate | RC Gate | 发布候选 | ≥85% | TPC-H SF=1 22/22 + QPS 基线 |
| G-Gate | GA Gate | 正式发布 | ≥85% | Point Select ≥10K QPS |

### v3.0.0 新增检查项（相比 v2.9.0）

| 检查项 | 说明 | 门禁 |
|--------|------|------|
| TPC-H SF=0.1 | TPC-H SF=0.1 22/22 可运行 | B-Gate |
| TPC-H SF=1 | TPC-H SF=1 22/22 可运行无 OOM | R-Gate |
| SQL Corpus ≥95% | SQL 兼容测试通过率 | R-Gate |
| Point Select QPS ≥10K | 性能及格线 | G-Gate |
| UPDATE QPS ≥5K | 写入性能及格线 | G-Gate |
| DELETE QPS ≥2K | 删除性能及格线 | G-Gate |
| Sysbench Gate | Point/UPDATE/INSERT 场景 QPS 对比 baseline | R-Gate |
| MySQL Protocol Test | mysql:5.7 容器握手测试 | R-Gate |

---

## 二、v3.0.0 量化目标

> 来源: `DEVELOPMENT_PLAN.md` §量化目标

| 指标 | v2.9.0 基线 | v3.0.0 及格线 | v3.0.0 目标线 | v3.1.0 卓越线 |
|------|-------------|--------------|--------------|--------------|
| Point Select QPS | ~2,000 | **≥10,000** | **≥20,000** | ≥50,000 |
| UPDATE QPS | ~950 | **≥5,000** | **≥10,000** | - |
| DELETE QPS | ~206 | **≥2,000** | **≥5,000** | - |
| TPC-H SF=1 | 22/22 (Q17/Q18 慢) | 22/22 可运行 | 22/22 p99<2s | - |
| SQL Corpus 通过率 | 92.6% | **≥95%** | **≥98%** | - |
| 覆盖率 | 84.18% | **≥85%** | ≥85% | - |

---

## 三、A-Gate (Alpha Gate)

### 3.1 入口条件

- Phase 0-4 开发任务完成
- 所有 Issue 关联的 PR 已合并
- 无 P0 Bug

### 3.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 |
|---|--------|----------|---------|
| A1 | 编译检查 | `cargo build --all-features --workspace` | 无错误 |
| A2 | 单元测试 | `cargo test --all-features --workspace` | ≥80% 通过 |
| A3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| A4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 |
| A5 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 无死链 |
| A6 | 覆盖率 | `cargo llvm-cov --all-features --lcov --output-path lcov.info` | ≥50% |
| A7 | 安全扫描 | `cargo audit` | 无高危漏洞 |

### 3.3 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥45% |
| optimizer | ≥40% |
| storage | ≥15% |
| catalog | ≥50% |
| parser | ≥50% |
| **整体** | **≥50%** |

### 3.4 证据格式

```json
{
  "gate": "A-GATE-v3.0.0",
  "commit": "<sha>",
  "status": "PASS|FAIL",
  "evidence": {
    "A1_build": {"command": "cargo build --all-features", "exit_code": 0},
    "A2_test": {"command": "cargo test --all-features", "passed": N, "failed": M, "exit_code": 0},
    "A3_clippy": {"command": "cargo clippy --all-features", "warnings": 0, "exit_code": 0},
    "A4_fmt": {"command": "cargo fmt --all -- --check", "diff_count": 0},
    "A5_docs": {"command": "bash scripts/gate/check_docs_links.sh", "broken_links": 0},
    "A6_coverage": {"total_pct": 52.3, "executor_pct": 46, "optimizer_pct": 41, "storage_pct": 16, "catalog_pct": 51, "parser_pct": 50},
    "A7_security": {"command": "cargo audit", "vulnerabilities": 0}
  }
}
```

---

## 四、B-Gate (Beta Gate)

### 4.1 入口条件

- A-Gate 已通过
- TPC-H SF=0.1 22/22 查询可运行（无 OOM）
- 无 P0/P1 Bug

### 4.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 |
|---|--------|----------|---------|
| B1 | 编译检查 | `cargo build --release --workspace` | 无错误 |
| B2 | 全量测试 | `cargo test --all-features` | ≥90% 通过 |
| B3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| B4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 |
| B5 | 覆盖率 | `cargo llvm-cov --all-features --lcov --output-path lcov.info` | ≥75% |
| B6 | 安全扫描 | `cargo audit` | 无高危漏洞 |
| B7 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 无死链 |
| B8 | TPC-H SF=0.1 | `scripts/gate/check_tpch.sh sf=0.1` | 22/22 通过，无 OOM |
| B9 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% |
| B10 | CBO Index Scan | `cargo test --test cbo_integration_test test_should_use_index` | 测试通过 |
| B11 | CBO Join Cost | `cargo test --test cbo_integration_test test_estimate_join_cost` | 测试通过 |
| B12 | CBO Optimizer | `cargo test -p sqlrustgo-optimizer` | 全部通过 |
| B13 | CBO Planner | `cargo test --test cbo_integration_test` | 全部通过 |

### 4.3 稳定性测试清单

| # | 检查项 | 命令/方法 | 通过标准 |
|---|--------|----------|---------|
| B-S1 | concurrency_stress_test | `cargo test --test concurrency_stress_test` | 全部通过 |
| B-S2 | crash_recovery_test | `cargo test --test crash_recovery_test` | 全部通过 |
| B-S3 | long_run_stability_test | `cargo test --test long_run_stability_test` | 全部通过 |
| B-S4 | wal_integration_test | `cargo test --test wal_integration_test` | 全部通过 |
| B-S5 | network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test` | 全部通过 |
| B-S6 | ssi_stress_test | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 |

### 4.4 TPC-H SF=0.1 详细要求

```bash
# 验证命令
cargo run -p sqlrustgo-bench-cli -- tpch-bench \
  --ddl scripts/tpch/tpch_schema.sql \
  --data <SF=0.1_DATA_PATH> \
  --queries all \
  --sf 0.1

# 通过标准
# - 22/22 查询全部可运行
# - 无 OOM 崩溃
# - Q17/Q18 允许慢，但必须出结果
```

### 4.5 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥60% |
| optimizer | ≥50% |
| storage | ≥20% |
| catalog | ≥60% |
| parser | ≥60% |
| **整体** | **≥75%** |

---

## 五、R-Gate (RC Gate)

### 5.1 入口条件

- B-Gate 已通过
- TPC-H SF=1 22/22 查询可运行（无 OOM）
- SQL Corpus ≥95%
- 无 P0/P1 Bug

### 5.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| R1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| R2 | Test | `cargo test --all-features` | 100% 通过 | `{passed, failed, exit_code}` |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| R4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| R5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥85% | `{total_pct, module_pcts}` |
| R6 | Security | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| R7 | Docs | `check_docs_links.sh` + R7b + R7c + R7d | 无死链/缺失/版本不一致 | `{broken_links, missing_docs}` |
| R8 | SQL Compat | `cargo test -p sqlrustgo-sql-corpus` | ≥95% | `{passed, total, pct}` |
| R9 | TPC-H SF=1 | `scripts/gate/check_tpch.sh sf=1` | 22/22 可运行 | `{passed, total, oom_count}` |
| R10 | Performance Baseline | `cargo bench && scripts/gate/check_perf_baseline.sh` | QPS 退化 ≤5% | `{baseline_path, delta_pct, pass}` |
| R11 | Sysbench Gate | `scripts/gate/check_sysbench.sh` | Point/UPDATE/INSERT 对比 baseline | `{point_qps, update_qps, insert_qps, delta}` |
| R12 | MySQL Protocol | `mysql:5.7` 容器握手测试 | 连接成功 | `{handshake, query_response}` |

### 5.3 R7 扩展说明

R7 包含四个子检查：

| 子项 | 检查内容 | 命令/方法 |
|------|----------|-----------|
| R7a | 死链检查 | `bash scripts/gate/check_docs_links.sh` |
| R7b | 必选文档存在性 | 检查 `docs/governance/VERSION_DOCS_SPEC.md` 定义的最小文档集 |
| R7c | 版本号一致性 | 所有文档头部版本号为 v3.0.0，无遗留旧版本号 |
| R7d | 文档与代码状态一致性 | 代码中标注的 feature 与文档描述匹配，Issue 引用有效 |

### 5.4 R10 性能回归检查规范

**基线文件**: `perf_baselines/v3.0.0/baseline.json`

**退化判定**:
- QPS 退化 ≤5% → PASS
- QPS 退化 5%-20% → 需人工解释
- QPS 退化 >20% → FAIL

**证据要求**: 必须包含 `baseline_path`, `delta_pct`, `pass` 三个字段

> **注意**: `perf_baselines/v3.0.0/baseline.json` 在 Alpha 阶段建立初版，RC 阶段更新。

### 5.5 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥75% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥70% |
| parser | ≥70% |
| **整体** | **≥85%** |

---

## 六、G-Gate (GA Gate)

### 6.1 入口条件

- R-Gate 已通过
- Point Select QPS ≥10,000
- UPDATE QPS ≥5,000
- DELETE QPS ≥2,000
- TPC-H SF=1 22/22 无 OOM
- 所有已知问题已关闭

### 6.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 |
|---|--------|----------|---------|
| G1 | Build | `cargo build --release --workspace` | 无错误 |
| G2 | Test | `cargo test --all-features` | 100% 通过 |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| G4 | Format | `cargo fmt --all -- --check` | 无格式错误 |
| G5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥85% |
| G6 | Security | `cargo audit` | 无漏洞 |
| G7 | Point Select QPS | `cargo bench -- point_select` | ≥10,000 QPS |
| G8 | UPDATE QPS | `cargo bench -- update_simple` | ≥5,000 QPS |
| G9 | DELETE QPS | `cargo bench -- delete_simple` | ≥2,000 QPS |
| G10 | TPC-H SF=1 | `scripts/gate/check_tpch.sh sf=1` | 22/22 通过，无 OOM |
| G11 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥98% |

### 6.3 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥80% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥75% |
| parser | ≥80% |
| **整体** | **≥85%** |

---

## 七、门禁状态追踪

### 7.1 各分支门禁要求

| 分支 | 门禁 | 覆盖率目标 | 性能要求 |
|------|------|-----------|---------|
| develop/v3.0.0 | A-Gate | ≥50% | 基线建立 |
| alpha/v3.0.0 | B-Gate | ≥75% | TPC-H SF=0.1 22/22 |
| beta/v3.0.0 | R-Gate | ≥85% | TPC-H SF=1 22/22 |
| rc/v3.0.0 | G-Gate | ≥85% | Point Select ≥10K |

### 7.2 当前状态 (v3.0.0)

| 门禁 | 状态 | 完成日期 | 备注 |
|------|------|----------|------|
| A-Gate | ⚪ 未启动 | TBD | 需 Phase 0-4 开发完成 |
| B-Gate | ⚪ 未启动 | TBD | 需 A-Gate 完成 |
| R-Gate | ⚪ 未启动 | TBD | 需 B-Gate 完成 |
| G-Gate | ⚪ 未启动 | TBD | 需 R-Gate 完成 |

---

## 八、CI Gate 新建清单

> 来源: `DEVELOPMENT_PLAN.md` §Phase 3 I-05

v3.0.0 需新建以下 CI Gate：

| Gate | 说明 | 触发条件 |
|------|------|---------|
| `tpch-gate` | TPC-H SF=0.1 全量，回归检测 | 每 PR |
| `sysbench-gate` | Point/UPDATE/INSERT 场景 QPS 对比 baseline | 每 PR |
| `coverage-trend` | 覆盖率趋势存储 + 下降告警（连续 3 次下降触发） | 每次 CI |
| `mysql-protocol-test` | mysql:5.7 容器握手测试 | 每次 CI |
| `chaos-gate-gitea` | 混沌工程 5 场景（迁移自 GitHub） | 每日 |

---

## 九、豁免规则

以下情况可申请门禁豁免：

| 豁免类型 | 条件 | 审批人 |
|----------|------|--------|
| 覆盖率豁免 | 新增代码可证明难以测试 | Tech Lead |
| 性能豁免 | 性能测试环境不稳定 | QA Lead |
| 文档豁免 | 文档更新不影响功能 | Docs Lead |
| TPC-H 豁免 | Q17/Q18 证明是存储层限制非查询逻辑错误 | Architect |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-06 | v3.0.0 初始版本：基于 gate_spec.md v1.2，添加 TPC-H/Sysbench/性能基线/Gate CI 新建清单 |

---

*本文档由 hermes-z6g4 维护。SSOT: gate_spec_v300.md 是 v3.0.0 门禁唯一权威来源。*
