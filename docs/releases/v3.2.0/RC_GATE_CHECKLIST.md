# v3.2.0 RC Gate Checklist

> **版本**: v3.2.0-rc-gate
> **创建日期**: 2026-05-15
> **维护人**: hermes-agent
> **阶段**: RC (Release Candidate)
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

---

## 一、门禁信息

### 1.1 门禁定义

| 属性 | 值 |
|------|-----|
| 门禁类型 | RC Gate |
| 执行日期 | TBD |
| 执行人 | TBD |
| 脚本 | `scripts/gate/check_rc_v320.sh` |
| 规范版本 | gate_spec_v320.md |

### 1.2 入口条件

- [ ] Beta Gate 24/24 PASS
- [ ] M1-M6 所有 P0/P1 功能已实现
- [ ] TPC-H SF=1 22/22 通过
- [ ] SQL Operations ≥85%
- [ ] L1 测试覆盖率 ≥80%

### 1.3 里程碑关联

| 里程碑 | 内容 | 状态 |
|--------|------|------|
| M7 | QPS 优化 + 内存管理 | TBD |
| M8 | RECURSIVE CTE + 冷存储 | TBD |
| RC1 | TPC-H SF=10 通过 | TBD |

---

## 二、Pre-Gate 自检清单

### 2.1 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build --release | `cargo build --release` | 编译成功 | ⬜ |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | ⬜ |
| cargo fmt | `cargo fmt --check` | 通过 | ⬜ |
| cargo test --lib | `cargo test --lib` | 全部通过 | ⬜ |

### 2.2 覆盖率检查

| 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|--------|------|----------|----------|------|
| L1 覆盖率 | `cargo llvm-cov` | ≥85% | TBD | ⬜ |

---

## 三、正式门禁检查 (RC)

### 3.1 核心检查 (R1-R16)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| R1 | Build | `cargo build --release` | 成功 | TBD | ⬜ |
| R2 | Test | `cargo test --lib` | ≥90% | TBD | ⬜ |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | TBD | ⬜ |
| R4 | Format | `cargo fmt --check` | 通过 | TBD | ⬜ |
| R5 | Coverage | `cargo llvm-cov` | ≥85% | TBD | ⬜ |
| R6 | Security | `cargo audit` | 无漏洞 | TBD | ⬜ |
| R7 | SQL Compat - MERGE | MERGE implementation | 通过 | TBD | ⬜ |
| R8 | SQL Compat - Event Scheduler | Event Scheduler | 通过 | TBD | ⬜ |
| R9 | GMP Workflow - State machine | State machine implementation | 通过 | TBD | ⬜ |
| R10 | GMP Mobile - Trusted collection protocol | Trusted collection protocol | 通过 | TBD | ⬜ |
| R11 | GMP SOP/Training - Binding check | Binding check | 通过 | TBD | ⬜ |
| R12 | GMP Device - Calibration management | Calibration management | 通过 | TBD | ⬜ |
| R13 | TPC-H SF=10 | Full dataset | 22/22 | TBD | ⬜ |
| R14 | Sysbench - point_select | point_select ≥ 30K QPS | ≥30K | TBD | ⬜ |
| R15 | Stability - 72h test | 72h stability | PASS | TBD | ⬜ |
| R16 | OO Documentation | All 13 docs exist | 13/13 | TBD | ⬜ |

### 3.2 稳定性测试 (R-S1~S16)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| R-S1 | concurrency_stress | `cargo test --test concurrency_stress_test` | PASS | TBD | ⬜ |
| R-S2 | crash_recovery | `cargo test --test crash_recovery_test` | PASS | TBD | ⬜ |
| R-S3 | long_run_stability | `cargo test --test long_run_stability_test` | PASS | TBD | ⬜ |
| R-S4 | wal_integration | `cargo test --test wal_integration_test` | PASS | TBD | ⬜ |
| R-S5 | network_tcp | `cargo test --test network_tcp_smoke_test` | PASS | TBD | ⬜ |
| R-S6 | ssi_stress | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | PASS | TBD | ⬜ |
| R-S7 | wal_crash_recovery | `cargo test -p sqlrustgo-server --test wal_crash_recovery_test` | PASS | TBD | ⬜ |
| R-S8 | audit_trail | `cargo test --test audit_trail_test` | PASS | TBD | ⬜ |
| R-S9 | gap_locking | `cargo test --test gap_locking_e2e_test` | PASS | TBD | ⬜ |
| R-S10 | digital_signature_test | `cargo test --test digital_signature_test` | PASS | TBD | ⬜ |
| R-S11 | immutable_record_test | `cargo test --test immutable_record_test` | PASS | TBD | ⬜ |
| R-S12 | correction_chain_test | `cargo test --test correction_chain_test` | PASS | TBD | ⬜ |
| R-S13 | provenance_tracking_test | `cargo test --test provenance_tracking_test` | PASS | TBD | ⬜ |
| R-S14 | workflow_engine_test | `cargo test --test workflow_engine_test` | PASS | TBD | ⬜ |
| R-S15 | trusted_timestamp_test | `cargo test --test trusted_timestamp_test` | PASS | TBD | ⬜ |
| R-S16 | hsm_integration_test | `cargo test --test hsm_integration_test` | PASS | TBD | ⬜ |

---

## 四、检查结果汇总

### 4.1 通过情况

| 类别 | 通过数 | 总数 | 通过率 |
|------|--------|------|--------|
| 核心检查 R1-R16 | TBD | 16 | TBD% |
| 稳定性测试 R-S1~S16 | TBD | 16 | TBD% |
| **总计** | **TBD** | **32** | **TBD%** |

### 4.2 RC Gate 最终结果

```
=== v3.2.0 RC Gate ===
R1:  Build ................... ⬜ TBD
R2:  Test (≥90%) ............ ⬜ TBD
R3:  Clippy .................. ⬜ TBD
R4:  Format .................. ⬜ TBD
R5:  Coverage (≥85%) ......... ⬜ TBD
R6:  Security Audit .......... ⬜ TBD
R7:  SQL Compat - MERGE ...... ⬜ TBD
R8:  SQL Compat - Event Sched.. ⬜ TBD
R9:  GMP Workflow - State ..... ⬜ TBD
R10: GMP Mobile - Trusted ..... ⬜ TBD
R11: GMP SOP/Training ......... ⬜ TBD
R12: GMP Device - Calibration.. ⬜ TBD
R13: TPC-H SF=10 (22/22) ..... ⬜ TBD
R14: Sysbench point_select .... ⬜ TBD
R15: Stability (72h) .......... ⬜ TBD
R16: OO Documentation ......... ⬜ TBD
R-S1~S16 ................... ⬜ TBD (0/16)

RC Gate: 0/32 PASS ⬜
RESULT: PENDING ⬜
```

---

## 五、失败项处理

### 5.1 失败项记录

| Issue | 描述 | 解决方案 | 状态 |
|-------|------|----------|------|
| 无 | - | - | - |

### 5.2 豁免申请

无

---

## 六、Post-Gate 收尾

### 6.1 文档更新

- [ ] 更新 BETA_GATE_CHECKLIST.md
- [ ] 更新 RC_GATE_CHECKLIST.md (本文档)
- [ ] 更新 DEVELOPMENT_PLAN.md
- [ ] 更新 TEST_PLAN.md
- [ ] 更新 CHANGELOG.md

### 6.2 分支操作

- [ ] 冻结 `rc/v3.2.0` 分支
- [ ] 创建 `release/v3.2.0` 分支
- [ ] 同步 `rc/v3.2.0` -> `release/v3.2.0`

---

## 七、审查与签名

| 角色 | 姓名 | 日期 | 签名 |
|------|------|------|------|
| 执行人 | TBD | TBD | ⬜ |
| 审查人 | TBD | TBD | ⬜ |

---

## 八、附录

### A.1 相关文档

- 门禁规范: `docs/governance/GATE_SPEC_MASTER.md`
- 测试计划: `docs/releases/v3.2.0/TEST_PLAN.md`
- 开发计划: `docs/releases/v3.2.0/DEVELOPMENT_PLAN.md`
- Beta Gate: `docs/releases/v3.2.0/BETA_GATE_CHECKLIST.md`

### A.2 RC Gate 脚本

- RC Gate 脚本: `scripts/gate/check_rc_v320.sh`

### A.3 M7-M8 里程碑交付物

| 里程碑 | 任务 | 交付物 |
|--------|------|--------|
| M7 | PERF-1 | Point SELECT QPS ≥1M ops/s |
| M7 | PERF-2 | TPC-H SF=10 22/22 通过 |
| M7 | PERF-5 | 内存优化 -15% |
| M8 | SQL-1 | RECURSIVE CTE 完整支持 |
| M8 | SQL-3 | 冷存储集成 S3/OSS |

### A.4 OO 文档清单 (13 文档)

| # | 文档 | 路径 |
|---|------|------|
| 1 | DIGITAL_SIGNATURE_CHAIN.md | `docs/releases/v3.2.0/oo/GMP/` |
| 2 | ELECTRONIC_SIGNATURE.md | `docs/releases/v3.2.0/oo/GMP/` |
| 3 | IMMUTABLE_RECORD.md | `docs/releases/v3.2.0/oo/GMP/` |
| 4 | CORRECTION_CHAIN.md | `docs/releases/v3.2.0/oo/GMP/` |
| 5 | PROVENANCE_TRACKING.md | `docs/releases/v3.2.0/oo/GMP/` |
| 6 | HSM_KMS_INTEGRATION.md | `docs/releases/v3.2.0/oo/GMP/` |
| 7 | GMP_WORKFLOW_ENGINE.md | `docs/releases/v3.2.0/oo/GMP/` |
| 8 | TRUSTED_TIMESTAMP.md | `docs/releases/v3.2.0/oo/GMP/` |
| 9 | CLUSTERED_INDEX.md | `docs/releases/v3.2.0/oo/` |
| 10 | MERGE_EXECUTION.md | `docs/releases/v3.2.0/oo/` |
| 11 | CBO_INTEGRATION.md | `docs/releases/v3.2.0/oo/` |
| 12 | GAP_LOCKING.md | `docs/releases/v3.2.0/oo/` |
| 13 | OO_ROADMAP.md | `docs/releases/v3.2.0/oo/` |

---

*最后更新: 2026-05-15*
