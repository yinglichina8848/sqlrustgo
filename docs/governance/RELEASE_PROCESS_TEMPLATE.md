# SQLRustGo 版本发布标准流程

> **版本**: v1.0
> **创建日期**: 2026-05-15
> **维护人**: hermes-z6g4

---

## 一、版本发布阶段流程

```
┌─────────────────────────────────────────────────────────────────┐
│  阶段入口检查 (Phase Entry Check)                                │
│  ├── 检查该阶段必需的文档是否存在                                 │
│  ├── 检查该阶段必需的测试脚本是否存在                              │
│  └── 检查 governance 规范要求的先决条件                            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  执行门禁检查 (Gate Check)                                       │
│  ├── 执行该阶段的所有门禁项目                                      │
│  ├── 记录每项检查的 PASS/FAIL/SKIP                               │
│  └── 失败项 → 创建 Issue → 修复 → PR → 验证                       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  门禁报告更新 (Gate Report Update)                                │
│  ├── 更新门禁清单的实际执行结果                                    │
│  ├── 添加 G9 等特殊情况说明                                       │
│  └── 标记门禁为 PASSED                                           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  分支同步 (Branch Sync)                                          │
│  ├── 将代码同步到下游分支                                         │
│  ├── rc → release → main (按阶段)                               │
│  └── 解决合并冲突                                                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                              ↓ (进入下一阶段)
```

---

## 二、阶段定义与入口条件

### 2.1 Alpha 阶段

**入口条件**:
- [ ] 开发分支 `alpha/v{VERSION}` 创建
- [ ] `ALPHA_GATE_CHECKLIST.md` 已创建
- [ ] 核心功能模块完成

**必需文档**:
- `ALPHA_GATE_CHECKLIST.md`

**门禁项目**: Alpha Gate (A1-A10)

### 2.2 Beta 阶段

**入口条件**:
- [ ] Alpha Gate 全部 PASS
- [ ] `BETA_GATE_CHECKLIST.md` 已创建
- [ ] `BETA_GATE_REPORT.md` 已创建 (Alpha 结果)
- [ ] 功能冻结 (Feature Freeze)

**必需文档**:
- `ALPHA_GATE_REPORT.md`
- `BETA_GATE_CHECKLIST.md`
- `BETA_GATE_REPORT.md`

**门禁项目**: Beta Gate (B1-B10, B-S1~S11)

### 2.3 RC 阶段

**入口条件**:
- [ ] Beta Gate 全部 PASS
- [ ] `RC_GATE_CHECKLIST.md` 已创建
- [ ] `RC_GATE_REPORT.md` 已创建 (Beta 结果)
- [ ] 代码冻结 (Code Freeze)
- [ ] 文档冻结 (Doc Freeze)

**必需文档**:
- `ALPHA_GATE_REPORT.md`
- `BETA_GATE_REPORT.md`
- `RC_GATE_CHECKLIST.md`
- `RC_GATE_REPORT.md`
- `SECURITY_REPORT.md`
- `PERFORMANCE_TARGETS.md`

**门禁项目**: RC Gate (R1-R12, R-S1~S5)

### 2.4 GA 阶段

**入口条件**:
- [ ] RC Gate 全部 PASS
- [ ] `GA_GATE_CHECKLIST.md` 已创建
- [ ] `GA_GATE_REPORT.md` 已创建 (RC 结果)
- [ ] 72小时稳定性测试通过
- [ ] WAL 崩溃恢复测试通过
- [ ] OO 闭环追踪已更新

**必需文档**:
- `ALPHA_GATE_REPORT.md`
- `BETA_GATE_REPORT.md`
- `RC_GATE_REPORT.md`
- `GA_GATE_CHECKLIST.md`
- `GA_GATE_REPORT.md`
- `USER_MANUAL.md`
- `API_REFERENCE.md`
- `RELEASE_NOTES.md`
- `CHANGELOG.md`
- `UPGRADE_GUIDE.md`
- `BENCHMARK.md`
- `TEST_REPORT.md`
- `SECURITY_ANALYSIS.md`

**门禁项目**: GA Gate (G1-G14, G-QA1~QA6, G-S1~S5, G-R1~R4)

---

## 三、门禁项目清单

### 3.1 Alpha Gate (A1-A10)

| # | 检查项 | 命令 | 期望结果 |
|---|--------|------|----------|
| A1 | Build | `cargo build --all-features` | 成功 |
| A2 | Test | `cargo test --lib` | 通过率 ≥90% |
| A3 | Clippy | `cargo clippy --all-features` | 零警告 |
| A4 | Format | `cargo fmt --check` | 通过 |
| A5 | Coverage | `cargo llvm-cov` | ≥75% |
| A6 | Security | `cargo audit` | 无高危漏洞 |
| A7 | SQL Compat | SQL Corpus | ≥70% |
| A8 | TPC-H SF=0.1 | `check_tpch.sh --sf0.1` | 22/22 |
| A9 | Docs | `check_docs_links.sh` | 无404 |
| A10 | Proofs | `check_proof.sh` | ≥10 |

### 3.2 Beta Gate (B1-B10, B-S1~S11)

| # | 检查项 | 命令 | 期望结果 |
|---|--------|------|----------|
| B1 | Build | `cargo build --all-features` | 成功 |
| B2 | Test | `cargo test --lib` | 通过率 ≥90% |
| B3 | Clippy | `cargo clippy --all-features` | 零警告 |
| B4 | Format | `cargo fmt --check` | 通过 |
| B5 | Coverage | `cargo llvm-cov` | ≥75% |
| B6 | Security | `cargo audit` | 无已知漏洞 |
| B7 | SQL Compat | SQL Corpus | ≥80% |
| B8 | TPC-H SF=1 | `check_tpch.sh --sf1` | 22/22 |
| B9 | Performance | `check_regression.sh` | 全部通过 |
| B10 | Proofs | `check_proof.sh` | ≥20 |
| B-S1 | Integration | `run_integration.sh` | 全部通过 |
| B-S2 | Sysbench | `check_sysbench.sh` | 4/4 |
| B-S3 | FTS | FTS tests | 全部通过 |
| B-S4 | GIS | GIS tests | 全部通过 |
| B-S5 | Event Scheduler | Event tests | 全部通过 |
| B-S6 | MySQL Protocol | MySQL tests | 全部通过 |
| B-S7 | WAL | WAL tests | 全部通过 |
| B-S8 | Recovery | Recovery tests | 全部通过 |
| B-S9 | Concurrency | Concurrency tests | 全部通过 |
| B-S10 | Auth | Auth tests | 全部通过 |
| B-S11 | Backup/Restore | Backup tests | 全部通过 |

### 3.3 RC Gate (R1-R12, R-S1~S5)

| # | 检查项 | 命令 | 期望结果 |
|---|--------|------|----------|
| R1 | Build | `cargo build --release` | 成功 |
| R2 | Test | `cargo test --all-features` | 100% |
| R3 | Clippy | `cargo clippy --all-features` | 零警告 |
| R4 | Format | `cargo fmt --check` | 通过 |
| R5 | Coverage | `cargo llvm-cov` | ≥85% |
| R6 | Security | `cargo audit` | 无已知漏洞 |
| R7 | SQL Compat | SQL Corpus | ≥80% |
| R8 | TPC-H SF=1 | `check_tpch.sh --sf1` | 22/22 |
| R9 | Performance | `check_regression.sh` | 全部通过 |
| R10 | Proofs | `check_proof.sh` | ≥30 |
| R11 | Docs | `check_docs_links.sh --all` | 无404 |
| R12 | MySQL Protocol | MySQL tests | 全部通过 |
| R-S1 | Integration | `run_integration.sh --full` | 全部通过 |
| R-S2 | Sysbench | `check_sysbench.sh` | 4/4 |
| R-S3 | FTS | FTS tests | 全部通过 |
| R-S4 | GIS | GIS tests | 全部通过 |
| R-S5 | Event Scheduler | Event tests | 全部通过 |

### 3.4 GA Gate (G1-G14, G-QA1~QA6, G-S1~S5, G-R1~R4)

| # | 检查项 | 命令 | 期望结果 |
|---|--------|------|----------|
| G1 | Build | `cargo build --release` | 成功 |
| G2 | Test | `cargo test --all-features` | 100% |
| G3 | Clippy | `cargo clippy --all-features` | 零警告 |
| G4 | Format | `cargo fmt --check` | 通过 |
| G5 | Coverage | `cargo llvm-cov` | ≥85% |
| G6 | Security | `cargo audit` | 无已知漏洞 |
| G7 | SQL Compat | SQL Corpus | ≥80% |
| G8 | TPC-H SF=1 | `check_tpch.sh --sf1` | 22/22 |
| G9 | Performance | `check_regression.sh` | 全部通过 |
| G10 | Proofs | `check_proof.sh` | ≥30 |
| G11 | Docs | `check_docs_links.sh --all` | 无404 |
| G12 | MySQL Protocol | MySQL tests | 全部通过 |
| G13 | Integration | `run_integration.sh --full` | 全部通过 |
| G14 | Stability (72h) | `long_run_stability_test` | 通过 |
| G-QA1 | Sqllogictest | QA enhancement | PASS |
| G-QA2 | Static Analysis | Miri | PASS |
| G-QA3 | Security Scan | audit | PASS |
| G-QA4 | Benchmark Std | Issue #863 | PASS |
| G-QA5 | Mutation Testing | Issue #864 | PASS |
| G-QA6 | CI/CD Integration | Issue #865 | PASS |
| G-S1 | Integration | `run_integration.sh --quick` | 全部通过 |
| G-S2 | WAL Crash | `wal_integration_test` | 通过 |
| G-S3 | FTS | FTS tests | 全部通过 |
| G-S4 | GIS | GIS tests | 全部通过 |
| G-S5 | Event Scheduler | Event tests | 全部通过 |
| G-R1 | Release Notes | `RELEASE_NOTES.md` | 完整 |
| G-R2 | Changelog | `CHANGELOG.md` | 完整 |
| G-R3 | Upgrade Guide | `UPGRADE_GUIDE.md` | 完整 |
| G-R4 | Binary Artifacts | build artifacts | 存在 |

---

## 四、Ignored 测试追踪

### 4.1 追踪要求

每个阶段必须检查并更新 `OO_DOCUMENT_ANALYSIS.md` 中的:
- 已知 ignore 测试数量
- 忽略原因分类
- 计划在下一版本解决的测试

### 4.2 分类标准

| 类别 | 说明 | 行动 |
|------|------|------|
| Feature Gap | 功能未实现 | 移到下一版本 |
| Known Issue | 已知问题 | 创建 Issue 追踪 |
| Performance | 性能问题 | 优化后解决 |
| Environment | 环境相关 | 标记条件 |

---

## 五、版本发布检查清单 (Pre-Release)

### 5.1 代码质量

- [ ] 所有 gate 通过
- [ ] 无 critical/high issue 开放
- [ ] 代码覆盖率达标
- [ ] 安全扫描通过

### 5.2 文档完整性

- [ ] 所有必需文档存在
- [ ] README.md 已更新
- [ ] CHANGELOG.md 已更新
- [ ] API 文档完整

### 5.3 测试完整性

- [ ] 无被忽略的核心功能测试
- [ ] 稳定性测试通过
- [ ] 回归测试通过
- [ ] 集成测试通过

### 5.4 OO 闭环追踪

- [ ] OO_DOCUMENT_ANALYSIS.md 已更新
- [ ] 已知问题已记录
- [ ] 下一版本计划已制定

### 5.5 发布准备

- [ ] Version 文件更新
- [ ] Release Notes 草稿完成
- [ ] 发布公告草稿完成
- [ ] 版本标签创建

---

## 六、分支管理策略

### 6.1 分支层级

```
main (生产环境)
  └── release/v{VERSION} (发布候选)
        └── rc/v{VERSION} (RC 开发)
              └── develop/v{VERSION} (主开发)
                    └── alpha/v{VERSION} (早期开发)
```

### 6.2 同步规则

| 阶段 | 同步方向 | 时机 |
|------|----------|------|
| Alpha → Beta | alpha → develop | Alpha PASS |
| Beta → RC | develop → rc | Beta PASS |
| RC → GA | rc → release → main | RC PASS |

---

*本文档由 hermes-z6g4 维护*
