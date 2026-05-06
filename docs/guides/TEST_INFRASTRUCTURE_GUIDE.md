# SQLRustGo 测试与 CI/CD 集成指南

> **版本**: 1.0
> **日期**: 2026-05-06
> **目标**: 帮助开发者和 AI Agent 理解、运行和扩展 SQLRustGo 的测试体系

---

## 一、测试层次

```
┌─────────────────────────────────────────────┐
│              CI/CD Gate (fast)               │
│  Unit Tests + Clippy + Fmt + Coverage Gate   │
├─────────────────────────────────────────────┤
│            Integration Tests                 │
│  SQL Corpus + TPC-H + SQLite Fuzz + QPS QoS  │
├─────────────────────────────────────────────┤
│             System Tests                     │
│  Chaos Engineering + Crash Recovery + E2E    │
├─────────────────────────────────────────────┤
│          Benchmark Tracking                  │
│  QPS Baseline + TPC-H + Perf Regression      │
└─────────────────────────────────────────────┘
```

---

## 二、底层：单元测试

```bash
# 运行所有单元测试
cargo test --all-features

# 运行单个 crate 的测试
cargo test -p sqlrustgo-parser
cargo test -p sqlrustgo-executor
cargo test -p sqlrustgo-storage

# 只运行单元测试（跳过集成测试）
cargo test --lib

# 覆盖率检查
bash scripts/gate/check_coverage.sh
```

### 门禁要求

| 阶段 | 覆盖率 | Parser | Executor | Storage |
|------|--------|--------|----------|---------|
| Beta | ≥75% | — | ≥60% | — |
| RC | ≥80% | — | ≥65% | ≥75% |
| **GA** | **≥85%** | **≥70%** | **≥70%** | **≥80%** |

---

## 三、中层：集成测试

### SQL Corpus

SQL Corpus 是一组标准化的 SQL 测试用例（位于 `sql_corpus/`），覆盖 DDL/DML/ADVANCED/PROCEDURES 等类别。

```bash
# 运行 SQL Corpus
cargo test -p sqlrustgo-sql-corpus

# 查看通过率
cargo test -p sqlrustgo-sql-corpus -- --nocapture 2>&1 | grep "passed"
```

### QPS 基准测试

QPS 基准测试测量各种操作类型的每秒查询数（MemoryStorage）。

```bash
# 运行全部 9 项基准（~5-10 分钟）
cargo test --test qps_benchmark_test -- --ignored --nocapture

# 运行单项基准
cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture
cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture
cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture
```

### TPC-H

TPC-H 需要外部数据文件（`~/sqlrustgo-tpch/data/*.tbl`）。

```bash
# 快速测试（Q1, Q6 仅）
bash scripts/gate/check_tpch.sh

# 需要数据生成
# ~/tpch-tools/dbgen/dbgen -s 0.1 -f -d -T a
```

### 模糊测试（Fuzz）

```bash
# 简单模糊测试（生成随机 SQL 并执行验证）
cargo run -p sqlrustgo-fuzz -- [iterations]
```

---

## 四、高层：门禁体系

### 门禁层次

| 门禁 | 脚本 | 阶段 | 内容 |
|------|------|------|------|
| R4 | `check_rc.sh` | Beta→RC | cargo test, integrated tests |
| R7 | `check_rc.sh` | Beta→RC | clippy, fmt |
| A1 | `check_rc.sh` | Beta→RC | SQL Corpus ≥85% |
| **R9** | **`check_regression.sh`** | **所有** | **QPS 回归检测 + E-09 底线** |
| R10 | `check_tpch.sh` | 可选 | TPC-H 扩展 |
| B1-B5 | `check_rc.sh` | Beta→RC | 覆盖率、形式化证明 |
| **GA** | **`check_ga.sh`** | **RC→GA** | **完整发布门禁** |
| S1 | `check_security.sh` | 所有 | 安全检查 |
| D1 | `check_docs_links.sh` | 所有 | 文档链接 |

### 运行门禁

```bash
# R9: 性能回归检测（推荐 --skip-run 跳过基准运行）
bash scripts/gate/check_regression.sh --skip-run

# RC 门禁
bash scripts/gate/check_rc.sh

# GA 门禁
bash scripts/gate/check_ga.sh
```

### E-09 硬性底线

R9 门禁中独立于回归检测的断言：
- **DELETE QPS ≥ 10,000**
- **UPDATE QPS ≥ 10,000**

---

## 五、基准与基线

### QPS 基线

| 文件 | 说明 |
|------|------|
| `perf_baselines/v2.9.0/baseline.json` | 9 项 QPS 基准值（MemoryStorage） |
| `perf_baselines/v2.9.0/current.json` | 最近一次实测 |
| `perf_baselines/v2.9.0/tpch_baseline.json` | TPC-H 参考值（SQLite） |

### 阈值体系

| 指标类别 | 指标 | PASS | WARN | FAIL |
|----------|------|------|------|------|
| **稳定** | simple/insert/update/delete/join/aggregation/order_by/complex_where | ≤5% | 5-20% | >20% |
| **环境敏感** | concurrent_select_8t | ≤5% | 5-30% | >30% |
| **E-09** | DELETE, UPDATE | ≥10K QPS | — | <10K |

---

## 六、CI/CD 集成

### Gitea Actions

已有工作流文件（`.gitea/workflows/`）：

| 工作流 | 触发条件 | 内容 |
|--------|----------|------|
| `ci.yml` | push/PR 到 develop/* | Hermes Pipeline: 编译 + 测试 + 门禁 + 形式化验证 |
| `gate-ci.yml` | push/PR 到 develop/* | B-Gate |
| `coverage-llvm-cov.yml` | workflow_dispatch | 覆盖率报告 |
| `coverage-tarpaulin.yml` | workflow_dispatch | Tarpaulin 覆盖率 |

### 新增 R9 到 CI

如需将 R9 性能回归检测加入 CI pipeline：

```yaml
# 在 ci.yml 的 postcheck 步骤后添加
- name: R9 Performance Regression
  run: |
    cd repo
    bash scripts/gate/check_regression.sh --skip-run
```

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| `scripts/gate/check_regression.sh` | R9 回归检测脚本 |
| `scripts/gate/check_tpch.sh` | R10 TPC-H 门禁 |
| `scripts/gate/check_ga.sh` | GA 最终门禁 |
| `docs/releases/v2.9.0/R9_PERFORMANCE_BASELINE_GUIDE.md` | R9 基线指南 |
| `docs/releases/v2.9.0/E08_QPS_BENCHMARK_REPORT.md` | QPS 基准报告 |
| `docs/releases/v2.9.0/ISSUE_CLOSURE_PLAN.md` | 遗留问题闭环计划 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-06*
