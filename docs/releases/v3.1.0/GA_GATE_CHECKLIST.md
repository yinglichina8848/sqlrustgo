# v3.1.0 GA Gate Checklist

> **版本**: v3.1.0-ga-gate
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **阶段**: GA (General Availability)
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

---

## 一、门禁信息

### 1.1 门禁定义

| 属性 | 值 |
|------|-----|
| 门禁类型 | GA Gate |
| 执行日期 | TBD |
| 执行人 | - |
| 脚本 | `scripts/gate/check_ga_v310.sh` |
| 规范版本 | gate_spec_v310.md |

### 1.2 入口条件

- [ ] RC Gate 18/19 PASS (R5 Coverage ≥85%)
- [ ] 所有 P0/P1 功能已实现
- [ ] SQL Operations ≥80%
- [ ] L1 测试覆盖率 ≥85%
- [ ] TPC-H SF=1 22/22 通过

---

## 二、Pre-Gate 自检清单

### 2.1 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build | `cargo build --all-features` | 编译成功 | ⏳ |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | ⏳ |
| cargo fmt | `cargo fmt --all -- --check` | 通过 | ⏳ |
| cargo test | `cargo test --lib` | 全部通过 | ⏳ |

### 2.2 覆盖率检查

| 检查项 | 命令 | 期望结果 | 当前 | 状态 |
|--------|------|----------|------|------|
| L1 覆盖率 | `cargo llvm-cov` | ≥85% | 81.65% | ⚠️ |

---

## 三、正式门禁检查 (GA)

### 3.1 核心检查 (G1-G12)

| # | 检查项 | 命令 | 期望结果 | 状态 |
|---|--------|------|----------|------|
| G1 | Build | `cargo build --all-features` | 成功 | ⏳ |
| G2 | Test | `cargo test --lib` | 100% | ⏳ |
| G3 | Clippy | `cargo clippy --all-features` | 零警告 | ⏳ |
| G4 | Format | `cargo fmt --all -- --check` | 通过 | ⏳ |
| G5 | Coverage | `cargo llvm-cov` | ≥85% | ⚠️ |
| G6 | Security | `cargo audit` | 无漏洞 | ⏳ |
| G7 | SQL Compat | SQL Corpus | ≥80% | ⏳ |
| G8 | TPC-H SF=1 | `check_tpch.sh --sf1` | 22/22 | ⏳ |
| G9 | Performance | `check_regression.sh` | 全部通过 | ⏳ |
| G10 | Proofs | `check_proof.sh` | ≥30 | ⏳ |
| G11 | Docs | `check_docs_links.sh --all` | 无404 | ⏳ |
| G12 | MySQL Protocol | `cargo test -p sqlrustgo-mysql-server` | 全部通过 | ⏳ |

### 3.2 QA 增强测试 (G-QA1~QA6)

| # | 检查项 | 命令 | 期望结果 | 状态 |
|---|--------|------|----------|------|
| G-QA1 | Sqllogictest Runner | `check_qa_enhancement.sh` | PASS | ⏳ |
| G-QA2 | Static Analysis (Miri) | `check_qa_enhancement.sh` | PASS | ⏳ |
| G-QA3 | Security Scan (audit) | `check_qa_enhancement.sh` | PASS | ⏳ |
| G-QA4 | Benchmark Standardization | `check_qa_enhancement.sh` | PASS | ⏳ |
| G-QA5 | Mutation Testing | `check_qa_enhancement.sh` | PASS | ⏳ |
| G-QA6 | CI/CD Quality Gate | `check_qa_enhancement.sh` | PASS | ⏳ |

### 3.3 扩展稳定性测试 (G-S1~S5)

| # | 检查项 | 命令 | 期望结果 | 状态 |
|---|--------|------|----------|------|
| G-S1 | Integration | `run_integration.sh --quick` | 全部通过 | ⏳ |
| G-S2 | Sysbench | `check_sysbench.sh ga` | 4/4 | ⏳ |
| G-S3 | FTS | `cargo test -p sqlrustgo-executor --test fts_tests` | 全部通过 | ⏳ |
| G-S4 | GIS | `cargo test --test gis_spatial_test` | 全部通过 | ⏳ |
| G-S5 | Event Scheduler | `cargo test --test event_scheduler_test` | 全部通过 | ⏳ |

### 3.4 发布检查 (G-R1~R4)

| # | 检查项 | 说明 | 状态 |
|---|--------|------|------|
| G-R1 | CHANGELOG | 更新到最新版本 | ⏳ |
| G-R2 | RELEASE_NOTES | 完成发布说明 | ⏳ |
| G-R3 | USER_MANUAL | 用户手册完整 | ⏳ |
| G-R4 | API_REFERENCE | API 文档完整 | ⏳ |

---

## 四、检查结果汇总

### 4.1 通过情况

| 类别 | 通过数 | 总数 | 通过率 |
|------|--------|------|--------|
| 核心检查 G1-G12 | 0 | 12 | 0% |
| QA 增强 G-QA1~QA6 | 0 | 6 | 0% |
| 稳定性测试 G-S1~S5 | 0 | 5 | 0% |
| 发布检查 G-R1~R4 | 0 | 4 | 0% |
| **总计** | **0** | **27** | **0%** |

### 4.2 GA Gate 当前状态

```
=== v3.1.0 GA Gate (待执行) ===
G1: Build ................... ⏳
G2: Test .................... ⏳
G3: Clippy .................. ⏳
G4: Format .................. ⏳
G5: Coverage (81.65%/85%) ... ⚠️ 待提升
G6: Security Audit .......... ⏳
G7: SQL Compat .............. ⏳
G8: TPC-H SF=1 ............. ⏳
G9: Performance ............. ⏳
G10: Proofs ................. ⏳
G11: Docs ................... ⏳
G12: MySQL Protocol ......... ⏳
G-S1~S5 ................... ⏳
G-R1~R4 ................... ⏳

GA Gate: 0/21 PASS (待执行)
```

---

## 五、失败项处理

### 5.1 当前已知问题

| Issue | 描述 | 解决方案 | 状态 |
|-------|------|----------|------|
| #916 | Coverage 81.65% < 85% | 增加测试覆盖 | 进行中 |

---

## 六、Post-Gate 收尾

### 6.1 发布前检查

- [ ] 创建 Git tag v3.1.0
- [ ] 创建 GitHub release
- [ ] 更新 Docker 镜像
- [ ] 更新文档网站
- [ ] 发布公告

### 6.2 分支操作

- [ ] 归档 `rc/v3.1.0`
- [ ] 创建 `archive/ga/v3.1.0`

---

## 七、审查与签名

| 角色 | 姓名 | 日期 | 签名 |
|------|------|------|------|
| 执行人 | - | - | - |
| 审查人 | - | - | - |

---

## 八、附录

### A.1 相关文档

- RC Gate Checklist: `RC_GATE_CHECKLIST.md`
- RC Gate Report: `RC_GATE_REPORT.md`
- 测试计划: `TEST_PLAN.md`
- 开发计划: `DEVELOPMENT_PLAN.md`

### A.2 门禁脚本

- GA Gate 脚本: `scripts/gate/check_ga_v310.sh`

---

*最后更新: 2026-05-14*
