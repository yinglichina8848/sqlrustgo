# v3.2.0 Beta Gate Checklist

> **版本**: v3.2.0-beta-gate
> **创建日期**: 2026-05-15
> **维护人**: hermes-agent
> **阶段**: Beta
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

---

## 一、门禁信息

### 1.1 门禁定义

| 属性 | 值 |
|------|-----|
| 门禁类型 | Beta Gate |
| 执行日期 | TBD |
| 执行人 | TBD |
| 脚本 | `scripts/gate/check_beta_v320.sh` |
| 规范版本 | gate_spec_v320.md |

### 1.2 入口条件

- [ ] Alpha Gate 13/13 PASS
- [ ] M1-M4 所有 P0 功能已实现
- [ ] TPC-H SF=1 22/22 可运行
- [ ] SQL Operations ≥80%
- [ ] L1 测试覆盖率 ≥90%

### 1.3 里程碑关联

| 里程碑 | 内容 | 状态 |
|--------|------|------|
| M5 | 电子签名 + 审计链验证 (GMP-2, OO-2) | TBD |
| M6 | Performance Schema 完善 (SQL-2) | TBD |

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
| L1 覆盖率 | `cargo llvm-cov` | ≥80% | TBD | ⬜ |

---

## 三、正式门禁检查 (Beta)

### 3.1 核心检查 (B1-B12)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| B1 | Build | `cargo build --release` | 成功 | TBD | ⬜ |
| B2 | L1 Test | `cargo test --lib` | ≥90% | TBD | ⬜ |
| B3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | TBD | ⬜ |
| B4 | Format | `cargo fmt --check` | 通过 | TBD | ⬜ |
| B5 | Coverage | `cargo llvm-cov` | ≥80% | TBD | ⬜ |
| B6 | Security | `cargo audit` | 无漏洞 | TBD | ⬜ |
| B7 | SQL Compat - Window Functions | NTILE/LEAD/LAG 测试 | 通过 | TBD | ⬜ |
| B8 | SQL Compat - Multi-table DML | UPDATE/DELETE JOIN | 通过 | TBD | ⬜ |
| B9 | SQL Compat - HASH JOIN | HASH JOIN 执行 | 通过 | TBD | ⬜ |
| B10 | TPC-H SF=1 | `check_tpch.sh --sf1` | 22/22 | TBD | ⬜ |
| B11 | GMP Digital Signature | sign/verify API | 通过 | TBD | ⬜ |
| B12 | GMP Electronic Signature | Policy engine | 通过 | TBD | ⬜ |

### 3.2 稳定性测试 (B-S1~S12)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| B-S1 | concurrency_stress | `cargo test --test concurrency_stress_test` | PASS | TBD | ⬜ |
| B-S2 | crash_recovery | `cargo test --test crash_recovery_test` | PASS | TBD | ⬜ |
| B-S3 | long_run_stability | `cargo test --test long_run_stability_test` | PASS | TBD | ⬜ |
| B-S4 | wal_integration | `cargo test --test wal_integration_test` | PASS | TBD | ⬜ |
| B-S5 | network_tcp | `cargo test --test network_tcp_smoke_test` | PASS | TBD | ⬜ |
| B-S6 | ssi_stress | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | PASS | TBD | ⬜ |
| B-S7 | wal_crash_recovery | `cargo test -p sqlrustgo-server --test wal_crash_recovery_test` | PASS | TBD | ⬜ |
| B-S8 | audit_trail | `cargo test --test audit_trail_test` | PASS | TBD | ⬜ |
| B-S9 | gap_locking | `cargo test --test gap_locking_e2e_test` | PASS | TBD | ⬜ |
| B-S10 | digital_signature_test | `cargo test --test digital_signature_test` | PASS | TBD | ⬜ |
| B-S11 | immutable_record_test | `cargo test --test immutable_record_test` | PASS | TBD | ⬜ |
| B-S12 | correction_chain_test | `cargo test --test correction_chain_test` | PASS | TBD | ⬜ |

---

## 四、检查结果汇总

### 4.1 通过情况

| 类别 | 通过数 | 总数 | 通过率 |
|------|--------|------|--------|
| 核心检查 B1-B12 | TBD | 12 | TBD% |
| 稳定性测试 B-S1~S12 | TBD | 12 | TBD% |
| **总计** | **TBD** | **24** | **TBD%** |

### 4.2 Beta Gate 最终结果

```
=== v3.2.0 Beta Gate ===
B1:  Build ................... ⬜ TBD
B2:  L1 Test (≥90%) .......... ⬜ TBD
B3:  Clippy .................. ⬜ TBD
B4:  Format .................. ⬜ TBD
B5:  Coverage (≥80%) ......... ⬜ TBD
B6:  Security Audit .......... ⬜ TBD
B7:  SQL Compat - Window Func.. ⬜ TBD
B8:  SQL Compat - Multi-table.. ⬜ TBD
B9:  SQL Compat - HASH JOIN... ⬜ TBD
B10: TPC-H SF=1 (22/22) ...... ⬜ TBD
B11: GMP Digital Signature .... ⬜ TBD
B12: GMP Electronic Signature.. ⬜ TBD
B-S1~S12 ................... ⬜ TBD (0/12)

Beta Gate: 0/24 PASS ⬜
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

- [ ] 更新 DEVELOPMENT_PLAN.md
- [ ] 更新 TEST_PLAN.md
- [ ] 更新 CHANGELOG.md

### 6.2 分支操作

- [ ] 创建 `beta/v3.2.0` 分支
- [ ] 归档 Alpha 相关分支

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
- Alpha Gate 报告: `docs/releases/v3.2.0/ALPHA_GATE_CHECKLIST.md`

### A.2 Beta Gate 脚本

- Beta Gate 脚本: `scripts/gate/check_beta_v320.sh`

### A.3 M5-M6 里程碑交付物

| 里程碑 | 任务 | 交付物 |
|--------|------|--------|
| M5 | GMP-2 | 电子签名 (21 CFR Part 11) |
| M5 | OO-2 | 电子签名设计文档 |
| M6 | SQL-2 | Performance Schema ≥60% |

---

*最后更新: 2026-05-15*
