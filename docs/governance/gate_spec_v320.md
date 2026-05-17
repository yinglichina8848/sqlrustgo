# v3.2.0 GA Gate 规范 (Gate Specification)

> **版本**: 1.0
> **创建日期**: 2026-05-18
> **维护人**: hermes-agent
> **适用版本**: v3.2.0
> **前置版本**: v3.1.0 GA

> **SSOT 声明**: `gate_spec_v320.md` 是 v3.2.0 GA 门禁定义的唯一权威来源。`RELEASE_POLICY.md`、`RELEASE_LIFECYCLE.md`、`AI_COLLABORATION.md` 等文档中的门禁描述仅作引用，不得独立定义门禁检查项。

---

## 一、门禁概述

v3.2.0 GA Gate 是正式发布前的最终质量门槛，确保：
1. 所有代码层检查通过（Build/Test/Clippy/Format/Coverage/Security）
2. 所有文档层检查通过（死链/必选文档/版本一致性/OO文档）
3. 所有性能层检查通过（Point/UPDATE/DELETE QPS/TPC-H/SQL Corpus）
4. 所有稳定性测试通过（B-S1~B-S20）
5. 所有流程合规检查通过（CI/Issue/Branch）

### GA Gate 入口条件

- R-Gate (R1-R16 + R-S1~R-S16) 已通过
- Point Select QPS ≥10,000
- UPDATE QPS ≥5,000
- DELETE QPS ≥2,000
- TPC-H SF=1 22/22 无 OOM
- SQL Corpus ≥98%
- 所有已知问题已关闭或有豁免记录

---

## 二、G1-G12 检查清单

### 2.1 代码层 Gate（G1-G6）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code, duration}` |
| G2 | Test | `cargo test --all-features` | **100% 通过** | `{passed, failed, exit_code}` |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| G4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| G5 | Coverage | `cargo llvm-cov test L1_CRATES --lib` | **≥85%** (全量) | `{total_pct, module_pcts}` |
| G6 | Security | `cargo audit` | 无高危漏洞 | `{vulnerabilities, exit_code}` |

### 2.2 功能层 Gate（G7-G12）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G7 | SQL Compat | `cargo test -p sqlrustgo-sql-corpus` | **≥85%** MySQL 语法 | `{passed, total, pct}` |
| G8 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh --sf1` | 22/22 PASS，无 OOM | `{passed, total, oom_count}` |
| G9 | Performance | Sysbench | 目标达成 | `{qps, threshold, pass}` |
| G10 | Formal Proofs | `bash scripts/gate/check_proof.sh` | **≥30 proofs** | `{count, verified_count}` |
| G11 | OO Docs | `bash scripts/gate/check_oo_docs.sh` | 全部存在 | `{missing_docs}` |
| G12 | MySQL Protocol | `cargo test -p sqlrustgo-mysql-server --test mysql_protocol_handshake_test` | 全部通过 | `{passed, failed}` |

---

## 三、QA Enhancement 测试（G-QA1~QA14）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G-QA1 | FTS | `cargo test --test fts_tests` | 全部通过 | `{passed, failed}` |
| G-QA2 | GIS | `cargo test --test gis_spatial_test` | 全部通过 | `{passed, failed}` |
| G-QA3 | Event Scheduler | `cargo test --test event_scheduler_test` | 全部通过 | `{passed, failed}` |
| G-QA4 | MERGE | `cargo test --test merge_execution_test` | 全部通过 | `{passed, failed}` |
| G-QA5 | Set Operations | `cargo test --test set_operation_test` | 全部通过 | `{passed, failed}` |
| G-QA6 | EXPLAIN ANALYZE | `cargo test --test explain_analyze_test` | 全部通过 | `{passed, failed}` |
| G-QA7 | DDL | `cargo test --test ddl_statement_test` | 全部通过 | `{passed, failed}` |
| G-QA8 | GMP Digital Signature | `cargo test -p sqlrustgo-gmp --test gmp_digital_signature_test` | 全部通过 | `{passed, failed}` |
| G-QA9 | GMP Electronic Signature | `cargo test -p sqlrustgo-gmp --test gmp_electronic_signature_test` | 全部通过 | `{passed, failed}` |
| G-QA10 | GMP Signature Algorithms | `cargo test -p sqlrustgo-gmp --test gmp_signature_algorithms_test` | 全部通过 | `{passed, failed}` |
| G-QA11 | GMP Signature Chain | `cargo test -p sqlrustgo-gmp --test gmp_signature_chain_test` | 全部通过 | `{passed, failed}` |
| G-QA12 | GMP Audit Chain Verify | `cargo test -p sqlrustgo-gmp --test gmp_audit_chain_verify_test` | 全部通过 | `{passed, failed}` |
| G-QA13 | GMP SOP Binding | `cargo test -p sqlrustgo-gmp --test gmp_sop_test` | 全部通过 | `{passed, failed}` |
| G-QA14 | QA Enhancement Suite | `bash scripts/gate/check_qa_enhancement.sh` | 全部通过 | `{passed, failed}` |

---

## 四、稳定性测试（G-S1~S20）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G-S1 | concurrency_stress | `cargo test --test concurrency_stress_test` | 全部通过 | `{passed, total}` |
| G-S2 | crash_recovery | `cargo test --test crash_recovery_test` | 全部通过 | `{passed, total}` |
| G-S3 | long_run_stability | `cargo test --test long_run_stability_test` | 全部通过 | `{passed, total}` |
| G-S4 | wal_integration | `cargo test --test wal_integration_test` | 全部通过 | `{passed, total}` |
| G-S5 | network_tcp | `cargo test --test network_tcp_smoke_test` | 全部通过 | `{passed, total}` |
| G-S6 | ssi_stress | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 | `{passed, total}` |
| G-S7 | audit_trail | `cargo test -p sqlrustgo-gmp --test gmp_audit_chain_verify_test` | 全部通过 | `{passed, total}` |
| G-S8 | integration_tests | `bash scripts/gate/run_integration.sh --quick` | 全部通过 | `{passed, failed}` |
| G-S9 | sysbench | `bash scripts/gate/check_sysbench.sh` | 目标 QPS | `{qps, threshold}` |
| G-S10 | regression_check | `bash scripts/gate/check_regression.sh` | 无性能退化 | `{regression_count}` |
| G-S11 | proof_count | `bash scripts/gate/check_proof.sh` | ≥30 formal proofs | `{count, verified_count}` |
| G-S12 | docs_links | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| G-S13 | sql_corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% | `{passed, total, pct}` |
| G-S14 | coverage_check | `cargo llvm-cov test L1_CRATES --lib` | ≥85% | `{total_pct}` |
| G-S15 | security_audit | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| G-S16 | static_analysis | `bash scripts/gate/check_static_analysis.sh` | Miri available | `{findings}` |
| G-S17 | mutants_check | `bash scripts/gate/check_mutants.sh` | 无存活 mutant | `{surviving}` |
| G-S18 | oo_docs | `bash scripts/gate/check_oo_docs.sh` | 全部存在 | `{missing_docs}` |
| G-S19 | information_schema | `cargo test --test information_schema_test --lib` | 全部通过 | `{passed, failed}` |
| G-S20 | benchmark_baseline | `bash scripts/gate/check_perf_baseline.sh` | 达成基线 | `{metrics}` |

---

## 五、通过条件

### 5.1 全部通过原则

所有门禁检查项必须 PASS，方可通过 GA Gate。

### 5.2 豁免条件

不满足的项必须在 `docs/governance/GATE_EXEMPTIONS.md` 有以下信息的豁免记录：
- 豁免 ID（格式 `EX-v320-XXX`）
- 豁免原因（不可抗力/基础设施缺失/外部依赖）
- Tech Lead 审批签字
- 复审日期
- 替代指标（如有）

### 5.3 无阻塞 OPEN Issue 原则

milestone `v3.2.0` 下不得有标记为 `GA-blocker` 的 OPEN Issue。

---

## 六、门禁失败处理

```
门禁 FAIL → 必须创建 Issue → 必须有修复 PR → 必须验证通过 → 才能关闭 Issue
```

禁止在未关闭相关 Issue 的情况下发布 GA。

---

## 七、L1 CRATES 定义

> **覆盖范围**: 以下 crate 的 `src/` 目录下所有 Rust 源文件的行覆盖率必须 ≥85%

```
sqlrustgo-executor
sqlrustgo-optimizer
sqlrustgo-planner
sqlrustgo-storage
sqlrustgo-transaction
sqlrustgo-server
sqlrustgo-parser
sqlrustgo-catalog
sqlrustgo-gmp
```

**注意**: `sqlrustgo-mysql-server` 和 `sqlrustgo-cli` 不在 L1 覆盖范围内。

---

*本文档由 hermes-agent 生成*
*最后更新: 2026-05-18*
