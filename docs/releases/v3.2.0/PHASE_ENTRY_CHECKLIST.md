# SQLRustGo v3.2.0 Phase Entry Checklist

> **版本**: v3.2.0-phase-entry
> **创建日期**: 2026-05-15
> **维护人**: hermes-agent
> **状态**: 规划中
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

---

## 一、文档概述

本文档定义 v3.2.0 版本所有阶段（Alpha、Beta、RC、GA）的入口检查清单。每个阶段的入口检查是强制性的，不通过检查不得进入下一阶段。

### 1.1 版本目标

```
v3.2.0 = "GMP Native 可信数据平台"
核心能力: 数字签名审计链、电子签名、EBR、工作流、HSM
```

### 1.2 里程碑映射

| 阶段 | 里程碑 | 目标日期 |
|------|--------|----------|
| Alpha | M1-M3 完成 | 2026-10-01 |
| Beta | M4-M6 完成 | 2026-12-01 |
| RC | M7-M8 + RC1 | 2027-01-15 |
| GA | GA Gate 23/23 | 2027-02-15 |

---

## 二、Alpha 阶段入口检查

### 2.1 门禁信息

| 属性 | 值 |
|------|-----|
| 门禁类型 | Alpha Gate |
| 执行日期 | (待填写) |
| 执行人 | (待填写) |
| 脚本 | `scripts/gate/check_alpha_v320.sh` |

### 2.2 入口条件

- [ ] 从 v3.1.0 GA 分支创建 `develop/v3.2.0` 分支
- [ ] 所有 P0 GMP 任务已规划并分配
- [ ] OO 文档目录结构已创建

### 2.3 开发进度检查 (M1-M3)

| 任务 ID | 功能 | 验收条件 | 状态 |
|---------|------|----------|------|
| GMP-1 | 数字签名审计链 | sign/verify API 完成 | (待检查) |
| GMP-6 | Trusted Timestamp | RFC3161 集成 | (待检查) |
| GMP-3 | Immutable Record | CREATE TABLE IMMUTABLE | (待检查) |
| GMP-4 | Correction Chain | CORRECT RECORD 语句 | (待检查) |
| GMP-5 | Provenance Tracking | 完整溯源链 | (待检查) |
| GMP-7 | 审计链验证工具 | 工具完成 | (待检查) |

### 2.4 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build | `cargo build --all-features` | 编译成功 | (待检查) |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | (待检查) |
| cargo fmt | `cargo fmt --all -- --check` | 通过 | (待检查) |
| cargo test (lib) | `cargo test --lib` | 全部通过 | (待检查) |

### 2.5 覆盖率检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| L0 覆盖率 | `cargo llvm-cov --lib` | ≥60% | (待检查) |

### 2.6 文档检查

| 检查项 | 要求 | 状态 |
|--------|------|------|
| OO-1 | 数字签名审计链设计文档存在 | (待检查) |
| OO-3 | Immutable Record 设计文档存在 | (待检查) |
| OO-4 | Correction Chain 设计文档存在 | (待检查) |
| OO-5 | Provenance Tracking 设计文档存在 | (待检查) |
| OO-8 | Trusted Timestamp 设计文档存在 | (待检查) |

### 2.7 Alpha 入口检查汇总

| 类别 | 检查项数 | PASS | FAIL | SKIP |
|------|----------|------|------|------|
| 入口条件 | 3 | (待填) | (待填) | (待填) |
| 开发进度 | 6 | (待填) | (待填) | (待填) |
| 代码质量 | 4 | (待填) | (待填) | (待填) |
| 覆盖率 | 1 | (待填) | (待填) | (待填) |
| 文档 | 5 | (待填) | (待填) | (待填) |
| **总计** | **19** | **(待填)** | **(待填)** | **(待填)** |

### 2.8 Alpha 入口结果

```
╔════════════════════════════════════════════════════════════╗
║  Alpha Gate 入口检查                                       ║
╠════════════════════════════════════════════════════════════╣
║  开发进度 (M1-M3): (待评估)                                 ║
║  代码覆盖率: ≥60%                                           ║
║  结果: (待填写)                                             ║
╚════════════════════════════════════════════════════════════╝
```

---

## 三、Beta 阶段入口检查

### 3.1 门禁信息

| 属性 | 值 |
|------|-----|
| 门禁类型 | Beta Gate |
| 执行日期 | (待填写) |
| 执行人 | (待填写) |
| 脚本 | `scripts/gate/check_beta_v320.sh` |
| 前置门禁 | Alpha Gate PASS |

### 3.2 入口条件

- [ ] Alpha Gate PASS (19/19 或豁免项已审批)
- [ ] M1-M4 所有任务已完成
- [ ] GMP 核心功能已实现

### 3.3 Alpha Gate 通过证明

| 检查项 | 要求 | 证据 | 状态 |
|--------|------|------|------|
| Alpha Gate 报告 | `docs/releases/v3.2.0/ALPHA_GATE_REPORT.md` 存在 | (待填) | (待检查) |
| Alpha Gate 结果 | PASS | (待填) | (待检查) |
| 豁免项 | 已在 `GATE_EXEMPTIONS.md` 审批 | (待填) | (待检查) |

### 3.4 开发进度检查 (M4)

| 任务 ID | 功能 | 验收条件 | 状态 |
|---------|------|----------|------|
| GMP-8 | HSM/KMS 集成 | TPM/HSM/KMS 支持 | (待检查) |
| OO-6 | HSM 集成设计文档 | 文档完成 | (待检查) |

### 3.5 GMP 核心功能检查

| 检查项 | 命令/方法 | 期望结果 | 状态 |
|--------|-----------|----------|------|
| 数字签名审计链 | `cargo test -p sqlrustgo-gmp` | GMP-1 相关测试 PASS | (待检查) |
| Immutable Record | `cargo test --test immutable_record_test` | 通过 | (待检查) |
| Correction Chain | `cargo test --test correction_chain_test` | 通过 | (待检查) |
| Provenance Tracking | `cargo test --test provenance_test` | 通过 | (待检查) |
| Trusted Timestamp | `cargo test --test timestamp_test` | 通过 | (待检查) |

### 3.6 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build | `cargo build --all-features` | 编译成功 | (待检查) |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | (待检查) |
| cargo fmt | `cargo fmt --all -- --check` | 通过 | (待检查) |
| cargo test | `cargo test --lib` | 全部通过 | (待检查) |

### 3.7 覆盖率检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| L1 覆盖率 | `cargo llvm-cov --lib` | ≥70% | (待检查) |

### 3.8 稳定性测试检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| 并发压力 | `cargo test --test concurrency_stress_test` | PASS | (待检查) |
| 崩溃恢复 | `cargo test --test crash_recovery_test` | PASS | (待检查) |
| 长时间运行 | `cargo test --test long_run_stability_test` | PASS | (待检查) |
| WAL 集成 | `cargo test --test wal_integration_test` | PASS | (待检查) |
| 网络 TCP | `cargo test --test network_tcp_smoke_test` | PASS | (待检查) |
| SSI 压力 | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | PASS | (待检查) |

### 3.9 Beta 入口检查汇总

| 类别 | 检查项数 | PASS | FAIL | SKIP |
|------|----------|------|------|------|
| 入口条件 | 3 | (待填) | (待填) | (待填) |
| Alpha 通过证明 | 3 | (待填) | (待填) | (待填) |
| 开发进度 (M4) | 2 | (待填) | (待填) | (待填) |
| GMP 核心功能 | 5 | (待填) | (待填) | (待填) |
| 代码质量 | 4 | (待填) | (待填) | (待填) |
| 覆盖率 | 1 | (待填) | (待填) | (待填) |
| 稳定性测试 | 6 | (待填) | (待填) | (待填) |
| **总计** | **24** | **(待填)** | **(待填)** | **(待填)** |

### 3.10 Beta 入口结果

```
╔════════════════════════════════════════════════════════════╗
║  Beta Gate 入口检查                                        ║
╠════════════════════════════════════════════════════════════╣
║  Alpha Gate: PASS                                          ║
║  M1-M4 完成度: (待评估)                                     ║
║  代码覆盖率: ≥70%                                           ║
║  稳定性测试: 6/6 PASS                                      ║
║  结果: (待填写)                                             ║
╚════════════════════════════════════════════════════════════╝
```

---

## 四、RC 阶段入口检查

### 4.1 门禁信息

| 属性 | 值 |
|------|-----|
| 门禁类型 | RC Gate |
| 执行日期 | (待填写) |
| 执行人 | (待填写) |
| 脚本 | `scripts/gate/check_rc_v320.sh` |
| 前置门禁 | Beta Gate PASS |

### 4.2 入口条件

- [ ] Beta Gate PASS (24/24 或豁免项已审批)
- [ ] M5-M8 所有任务已完成
- [ ] 所有 P0 功能已实现
- [ ] TPC-H SF=1 22/22 通过

### 4.3 Beta Gate 通过证明

| 检查项 | 要求 | 证据 | 状态 |
|--------|------|------|------|
| Beta Gate 报告 | `docs/releases/v3.2.0/BETA_GATE_REPORT.md` 存在 | (待填) | (待检查) |
| Beta Gate 结果 | PASS | (待填) | (待检查) |
| 豁免项 | 已在 `GATE_EXEMPTIONS.md` 审批 | (待填) | (待检查) |

### 4.4 开发进度检查 (M5-M8)

| 任务 ID | 功能 | 验收条件 | 状态 |
|---------|------|----------|------|
| GMP-2 | 电子签名 | 21 CFR Part 11 合规 | (待检查) |
| OO-2 | 电子签名设计文档 | 文档完成 | (待检查) |
| SQL-2 | Performance Schema | ≥60% 覆盖率 | (待检查) |
| PERF-1 | Point SELECT QPS | ≥1M ops/s | (待检查) |
| PERF-2 | TPC-H SF=10 | 22/22 通过 | (待检查) |
| PERF-5 | 内存优化 | -15% 占用 | (待检查) |
| SQL-1 | RECURSIVE CTE | 完整支持 | (待检查) |
| SQL-3 | 冷存储集成 | S3/OSS | (待检查) |

### 4.5 P0 功能完整性检查

| 任务 ID | 功能 | 验收条件 | 状态 |
|---------|------|----------|------|
| GMP-1 | 数字签名审计链 | sign/verify API | (待检查) |
| GMP-2 | 电子签名 | 21 CFR Part 11 | (待检查) |
| GMP-3 | Immutable Record | CREATE TABLE IMMUTABLE | (待检查) |
| GMP-4 | Correction Chain | CORRECT RECORD | (待检查) |
| GMP-5 | Provenance Tracking | 完整溯源链 | (待检查) |
| GMP-6 | Trusted Timestamp | RFC3161 | (待检查) |
| GMP-7 | 审计链验证工具 | 工具完成 | (待检查) |
| GMP-8 | HSM/KMS 集成 | TPM/HSM/KMS | (待检查) |

### 4.6 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build | `cargo build --release --all-features` | 编译成功 | (待检查) |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | (待检查) |
| cargo fmt | `cargo fmt --all -- --check` | 通过 | (待检查) |
| cargo test | `cargo test --all-features` | 全部通过 | (待检查) |

### 4.7 覆盖率检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| L1 覆盖率 | `cargo llvm-cov --all-features --lib` | ≥75% | (待检查) |

### 4.8 性能基准检查

| 检查项 | 命令/方法 | 期望结果 | 状态 |
|--------|-----------|----------|------|
| Point SELECT QPS | `cargo bench -- point_select` | ≥1M ops/s | (待检查) |
| TPC-H SF=10 | `check_tpch.sh --sf10` | 22/22 | (待检查) |
| 内存占用 | 对比 v3.1.0 | -15% | (待检查) |
| 并发连接 | `cargo test --test concurrency_stress_test` | 200+ | (待检查) |

### 4.9 稳定性测试检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| 并发压力 | `cargo test --test concurrency_stress_test` | PASS | (待检查) |
| 崩溃恢复 | `cargo test --test crash_recovery_test` | PASS | (待检查) |
| 长时间运行 | `cargo test --test long_run_stability_test` | PASS | (待检查) |
| WAL 集成 | `cargo test --test wal_integration_test` | PASS | (待检查) |
| 网络 TCP | `cargo test --test network_tcp_smoke_test` | PASS | (待检查) |
| SSI 压力 | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | PASS | (待检查) |
| Gap Locking | `cargo test --test gap_locking_e2e_test` | PASS | (待检查) |
| 集合操作 | `cargo test --test set_operation_test` | PASS | (待检查) |
| 窗口函数 | `cargo test --test window_function_boundary_test` | PASS | (待检查) |

### 4.10 SQL 兼容性检查

| 检查项 | 命令/方法 | 期望结果 | 状态 |
|--------|-----------|----------|------|
| SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥80% | (待检查) |
| RECURSIVE CTE | `cargo test --test recursive_cte_test` | 通过 | (待检查) |

### 4.11 安全扫描检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo audit | `cargo audit` | 无高危漏洞 | (待检查) |

### 4.12 RC 入口检查汇总

| 类别 | 检查项数 | PASS | FAIL | SKIP |
|------|----------|------|------|------|
| 入口条件 | 4 | (待填) | (待填) | (待填) |
| Beta 通过证明 | 3 | (待填) | (待填) | (待填) |
| 开发进度 (M5-M8) | 8 | (待填) | (待填) | (待填) |
| P0 功能完整性 | 8 | (待填) | (待填) | (待填) |
| 代码质量 | 4 | (待填) | (待填) | (待填) |
| 覆盖率 | 1 | (待填) | (待填) | (待填) |
| 性能基准 | 4 | (待填) | (待填) | (待填) |
| 稳定性测试 | 9 | (待填) | (待填) | (待填) |
| SQL 兼容性 | 2 | (待填) | (待填) | (待填) |
| 安全扫描 | 1 | (待填) | (待填) | (待填) |
| **总计** | **44** | **(待填)** | **(待填)** | **(待填)** |

### 4.13 RC 入口结果

```
╔════════════════════════════════════════════════════════════╗
║  RC Gate 入口检查                                          ║
╠════════════════════════════════════════════════════════════╣
║  Beta Gate: PASS                                          ║
║  M5-M8 完成度: (待评估)                                    ║
║  P0 功能: 8/8 完成                                        ║
║  代码覆盖率: ≥75%                                          ║
║  TPC-H SF=10: 22/22                                       ║
║  结果: (待填写)                                            ║
╚════════════════════════════════════════════════════════════╝
```

---

## 五、GA 阶段入口检查

### 5.1 门禁信息

| 属性 | 值 |
|------|-----|
| 门禁类型 | GA Gate |
| 执行日期 | (待填写) |
| 执行人 | (待填写) |
| 脚本 | `scripts/gate/check_ga_v320.sh` |
| 前置门禁 | RC Gate PASS |

### 5.2 入口条件

- [ ] RC Gate PASS (44/44 或豁免项已审批)
- [ ] GA-1 到 GA-10 所有检查项已验证
- [ ] Formal proofs ≥30 个
- [ ] GMP 合规验证通过
- [ ] 综合评分 ≥80/100

### 5.3 RC Gate 通过证明

| 检查项 | 要求 | 证据 | 状态 |
|--------|------|------|------|
| RC Gate 报告 | `docs/releases/v3.2.0/RC_GATE_REPORT.md` 存在 | (待填) | (待检查) |
| RC Gate 结果 | PASS | (待填) | (待检查) |
| 豁免项 | 已在 `GATE_EXEMPTIONS.md` 审批 | (待填) | (待检查) |

### 5.4 GA 门禁检查 (GA-1 到 GA-10)

| # | 检查项 | 命令/方法 | 通过标准 | 状态 |
|---|--------|-----------|----------|------|
| GA-1 | Release Build | `cargo build --release --workspace` | 成功 | (待检查) |
| GA-2 | 测试 100% | `cargo test --all-features` | 0 failures | (待检查) |
| GA-3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | (待检查) |
| GA-4 | Format | `cargo fmt --all -- --check` | 通过 | (待检查) |
| GA-5 | 覆盖率 | `cargo llvm-cov --all-features --lib` | ≥80% | (待检查) |
| GA-6 | 安全扫描 | `cargo audit` | 无漏洞 | (待检查) |
| GA-7 | GMP 合规 | 电子签名 + 审计链验证 | 通过 | (待检查) |
| GA-8 | TPC-H SF=10 | `check_tpch.sh --sf10` | 22/22 | (待检查) |
| GA-9 | QPS 基准 | `cargo bench` | 全部 ≥目标值 | (待检查) |
| GA-10 | Formal proofs | TLA+ 证明 | ≥30 个 | (待检查) |

### 5.5 GMP 合规验证

| 检查项 | 验证方法 | 通过标准 | 状态 |
|--------|----------|----------|------|
| 数字签名审计链 | 功能测试 + 安全审计 | 完整链验证 | (待检查) |
| 电子签名 | 21 CFR Part 11 合规检查 | 合规 | (待检查) |
| Immutable Record | 禁止 DML 验证 | 通过 | (待检查) |
| Correction Chain | 修正链完整性验证 | 通过 | (待检查) |
| Provenance Tracking | 溯源链完整性验证 | 通过 | (待检查) |
| HSM/KMS 集成 | TPM/HSM/KMS 功能测试 | 通过 | (待检查) |

### 5.6 性能验收

| 指标 | 目标 | 实际值 | 状态 |
|------|------|--------|------|
| Point SELECT QPS | ≥1,000,000 ops/s | (待填) | (待检查) |
| Complex WHERE QPS | ≥500,000 ops/s | (待填) | (待检查) |
| INSERT QPS | ≥800,000 ops/s | (待填) | (待检查) |
| UPDATE QPS | ≥800,000 ops/s | (待填) | (待检查) |
| DELETE QPS | ≥800,000 ops/s | (待填) | (待检查) |
| TPC-H SF=10 | 22/22 通过 | (待填) | (待检查) |
| 内存占用 | ≤85% (较 v3.1.0 -15%) | (待填) | (待检查) |
| 死锁检测延迟 | <50ms | (待填) | (待检查) |

### 5.7 MySQL 兼容性评估

| 维度 | 目标 | 实际值 | 状态 |
|------|------|--------|------|
| SQL 语言 | 90/100 | (待填) | (待检查) |
| 存储引擎 | 80/100 | (待填) | (待检查) |
| 可观测性 | 75/100 | (待填) | (待检查) |
| 安全 | 90/100 | (待填) | (待检查) |
| 高可用 | 75/100 | (待填) | (待检查) |
| **总体** | **≥80/100** | **(待填)** | **(待检查)** |

### 5.8 文档完整性检查

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 版本发布说明 | `RELEASE_NOTES.md` 存在 | (待检查) |
| 用户指南 | `docs/user/USER_MANUAL.md` 更新 | (待检查) |
| 迁移指南 | `docs/standard/templates/MIGRATION_GUIDE.md` 存在 | (待检查) |
| API 文档 | `docs/api/` 更新 | (待检查) |
| OO 文档索引 | `docs/releases/v3.2.0/oo/README.md` 存在 | (待检查) |

### 5.9 GA 入口检查汇总

| 类别 | 检查项数 | PASS | FAIL | SKIP |
|------|----------|------|------|------|
| 入口条件 | 5 | (待填) | (待填) | (待填) |
| RC 通过证明 | 3 | (待填) | (待填) | (待填) |
| GA 门禁 (GA-1~10) | 10 | (待填) | (待填) | (待填) |
| GMP 合规验证 | 6 | (待填) | (待填) | (待填) |
| 性能验收 | 8 | (待填) | (待填) | (待填) |
| MySQL 兼容性 | 6 | (待填) | (待填) | (待填) |
| 文档完整性 | 5 | (待填) | (待填) | (待填) |
| **总计** | **43** | **(待填)** | **(待填)** | **(待填)** |

### 5.10 GA 入口结果

```
╔════════════════════════════════════════════════════════════╗
║  GA Gate 入口检查                                          ║
╠════════════════════════════════════════════════════════════╣
║  RC Gate: PASS                                            ║
║  GA 门禁 (GA-1~10): (待评估)                               ║
║  GMP 合规: (待验证)                                        ║
║  代码覆盖率: ≥80%                                         ║
║  Formal proofs: ≥30 个                                    ║
║  综合评分: ≥80/100                                        ║
║  结果: (待填写)                                            ║
╚════════════════════════════════════════════════════════════╝
```

---

## 六、通用检查规则

### 6.1 通过标准

```
门禁通过 = (所有 MANDATORY 项 PASS) + (所有 FAIL 项有 Issue/PR) + (所有豁免项已审批)
```

### 6.2 检查结果分类

| 结果 | 说明 | 处理方式 |
|------|------|----------|
| PASS | 完全满足 | 进入下一阶段 |
| FAIL | 不满足 | 必须修复或申请豁免 |
| SKIP | 条件不满足 | 需要人工判断 |
| N/A | 不适用 | 记录原因 |

### 6.3 禁止的模式

```
❌ 门禁 FAIL → 跳过 → 合并代码 → 问题丢失
❌ Issue 已创建 → 未关联 PR → 无人追踪
❌ 检查通过 → 未记录证据 → 后续无法复现
❌ 豁免未申请 → 直接忽略 → 违反流程
```

### 6.4 正确的模式

```
✅ 门禁检查 → 记录结果 → FAIL → 创建 Issue → 修复 PR → 验证 PASS → 关闭 Issue
✅ 门禁检查 → 记录结果 → FAIL → 评估豁免 → 申请审批 → 记录到 GATE_EXEMPTIONS.md
✅ 门禁通过 → 记录证据 → 发布报告 → 更新 milestone → 通知团队
```

---

## 七、附录

### 7.1 相关文档

| 文档 | 路径 |
|------|------|
| 门禁规范 | `docs/governance/GATE_SPEC_MASTER.md` |
| 门禁模版 | `docs/governance/GATE_CHECKLIST_TEMPLATE.md` |
| 开发计划 | `docs/releases/v3.2.0/DEVELOPMENT_PLAN.md` |
| Alpha 门禁报告 | `docs/releases/v3.2.0/ALPHA_GATE_REPORT.md` |
| Beta 门禁报告 | `docs/releases/v3.2.0/BETA_GATE_REPORT.md` |
| RC 门禁报告 | `docs/releases/v3.2.0/RC_GATE_REPORT.md` |

### 7.2 门禁脚本

| 阶段 | 脚本路径 |
|------|----------|
| Alpha | `scripts/gate/check_alpha_v320.sh` |
| Beta | `scripts/gate/check_beta_v320.sh` |
| RC | `scripts/gate/check_rc_v320.sh` |
| GA | `scripts/gate/check_ga_v320.sh` |

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-15*
