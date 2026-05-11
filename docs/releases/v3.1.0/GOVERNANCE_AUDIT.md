# v3.1.0 版本治理审计

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **状态**: 🟡 规划中  
> **目标**: 确保 v3.1.0 符合 Harness 约束原则和 Governance 规范

---

## 一、审计依据

| 规范文档 | 用途 |
|---------|------|
| `docs/governance/gate_spec_v300.md` | 门禁规范 SSOT |
| `docs/governance/VERSION_DOCS_SPEC.md` | 版本文档规范 |
| `docs/governance/DOCUMENT_COMPLETENESS_CHECK.md` | 文档完整性检查 |
| `docs/governance/GATE_EXEMPTIONS.md` | 门禁豁免记录 |
| `docs/governance/RELEASE_POLICY.md` | 发布策略 |
| `docs/governance/RELEASE_LIFECYCLE.md` | 版本生命周期 |
| `docs/governance/AI_COLLABORATION.md` | AI 协作规则 |

---

## 二、v3.1.0 文档清单

### 2.1 必需文档（VERSION_DOCS_SPEC）

| 文档 | 状态 | 负责人 | 截止日期 |
|------|------|--------|----------|
| `RELEASE_NOTES.md` | ✅ | Hermes | 2026-05-11 |
| `CHANGELOG.md` | ✅ | Hermes | 2026-05-11 |
| `FEATURE_MATRIX.md` | ✅ | Hermes | 2026-05-11 |
| `TEST_PLAN.md` | ✅ | Hermes | 2026-05-11 |
| `PERFORMANCE_TARGETS.md` | ✅ | Hermes | 2026-05-11 |
| `INSTALL.md` | ✅ | Hermes | 2026-05-11 |
| `DEPLOYMENT_GUIDE.md` | ✅ | Hermes | 2026-05-11 |
| `QUICK_START.md` | ✅ | Hermes | 2026-05-11 |
| `COVERAGE_REPORT.md` | ⏳ 待生成 | — | RC 前 |
| `SECURITY_ANALYSIS.md` | ⏳ 待生成 | — | RC 前 |
| `MIGRATION_GUIDE.md` | ⏳ 待生成 | — | RC 前 |
| `DEVELOPMENT_PLAN.md` | ✅ | Hermes | 2026-05-11 |
| `GMP_COMPLIANCE_ROADMAP.md` | ✅ | Hermes | 2026-05-11 |
| `ARCHITECTURE_RECONSTRUCTION_PLAN.md` | ✅ | Hermes | 2026-05-11 |
| `COVERAGE_TEST_IMPROVEMENT_PLAN.md` | ✅ | Hermes | 2026-05-11 |

### 2.2 治理文档更新

| 文档 | 任务 | 状态 |
|------|------|------|
| `GATE_EXEMPTIONS.md` | 添加 v3.1.0 豁免记录（如有） | ⏳ 待定 |
| `gate_lifecycle_tracking.md` | 添加 v3.1.0 追踪记录 | ⏳ 待定 |
| `gate_spec_v300.md` | 评估是否需要升级为 v3.1.0 规范 | ⏳ 待定 |

---

## 三、门禁规范对照

### 3.1 Beta Gate 阈值

| 检查项 | v3.0.0 阈值 | v3.1.0 阈值 | 变化 |
|--------|-------------|-------------|------|
| 测试通过率 | ≥90% | ≥90% | — |
| 覆盖率 | ≥50% | ≥50% | — |
| SQL Corpus | ≥80% | ≥80% | — |
| TPC-H | SF=0.1 | SF=0.1 | — |
| Clippy | 零警告 | 零警告 | — |
| Format | 通过 | 通过 | — |
| Stability B-S1~B-S5 | ≥95% | 全部 PASS | 收紧 |

### 3.2 RC Gate 阈值

| 检查项 | v3.0.0 阈值 | v3.1.0 阈值 | 变化 |
|--------|-------------|-------------|------|
| 测试通过率 | 100% | 100% | — |
| 覆盖率 | ≥60% | ≥60% | — |
| SQL Corpus | ≥95% | ≥95% | — |
| TPC-H | SF=1 | SF=1 | — |
| Performance Baseline | 存在 | 存在 | — |
| Formal Proofs | ≥10 | ≥10 | — |

### 3.3 GA Gate 阈值

| 检查项 | v3.0.0 阈值 | v3.1.0 阈值 | 变化 |
|--------|-------------|-------------|------|
| 覆盖率 | ≥22%（本地） | ≥65% | 收紧 |
| SQL Corpus | ≥95% | ≥98% | 收紧 |
| Point SELECT QPS | ≥10K | ≥10K | — |
| UPDATE QPS | ≥5K | ≥5K | — |
| DELETE QPS | ≥2K | ≥2K | — |
| QPS Benchmark | 8/8 within 5% | 8/8 within 5% | — |
| Formal Proofs | ≥10 | ≥30 | 新增 |
| INFORMATION_SCHEMA | 未检查 | ≥80% | 新增 |
| CBO Index | 未检查 | PASS | 新增 |

---

## 四、Issue 追踪要求

### 4.1 Issue 创建规则

依据 `docs/governance/ISSUE_CLOSING_VERIFICATION.md`：
- 每个门禁失败项必须创建 Gitea Issue
- Issue 必须关联 milestone（v3.1.0-alpha/beta/rc/ga）
- 关闭前必须验证 `closedByPullRequestsReferences` 非空

### 4.2 v3.1.0 Issue 预估

| 类别 | 数量 | 优先级 |
|------|------|--------|
| P0 功能项 | ~8 | P0 |
| P1 功能项 | ~10 | P1 |
| 测试/覆盖率 | ~6 | P1 |
| 文档项 | ~12 | P1-P2 |
| **总计** | ~36 | |

---

## 五、合规检查清单

### 5.1 Alpha → Beta 转换检查

- [ ] 所有 P0 Issue 已分配
- [ ] Alpha 分支 `alpha/v3.1.0` 已创建
- [ ] Alpha 门禁全部 PASS
- [ ] 门禁脚本 `check_beta_v310.sh` 已验证
- [ ] `gate_lifecycle_tracking.md` 已更新

### 5.2 Beta → RC 转换检查

- [ ] Beta 门禁 14/14 PASS
- [ ] 所有 P1 Issue 已关闭或推迟
- [ ] `beta/v3.1.0` 分支已创建
- [ ] `rc/v3.1.0` 分支已创建
- [ ] `gate_lifecycle_tracking.md` 已更新

### 5.3 RC → GA 转换检查

- [ ] RC 门禁 16/16 PASS
- [ ] 所有 P0/P1 Issue 已关闭
- [ ] 发布文档 8 份全部存在
- [ ] 版本号一致性检查通过
- [ ] GitHub release/v3.1.0 已同步
- [ ] Tag v3.1.0 已推送
- [ ] 所有 mirror 已同步

---

## 六、豁免申请流程

如需申请豁免（在 `GATE_EXEMPTIONS.md` 登记）：

```
条件：
1. 门禁失败项影响范围有限
2. 有明确的修复计划和时间表
3. Architect 批准

流程：
1. 创建 Gitea Issue 说明豁免理由
2. Architect 审批
3. 在 GATE_EXEMPTIONS.md §v3.1.0 登记
4. 关联 Issue 和 milestone
```

---

## 七、相关文档索引

| 文档 | 用途 |
|------|------|
| `docs/releases/v3.1.0/DEVELOPMENT_PLAN.md` | 开发计划 |
| `docs/governance/gate_spec_v300.md` | 门禁规范 |
| `docs/governance/VERSION_DOCS_SPEC.md` | 文档规范 |
| `scripts/gate/check_beta_v310.sh` | Beta 门禁脚本 |
| `scripts/gate/check_rc_v310.sh` | RC 门禁脚本 |
| `scripts/gate/check_ga_v310.sh` | GA 门禁脚本 |

---

*本文档由 hermes agent 创建，用于追踪 v3.1.0 治理合规状态。*
*每次 Beta/RC/GA Gate 检查后更新。*
