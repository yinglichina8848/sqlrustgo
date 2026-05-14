# v3.1.0 GA Gate 规范 (Gate Specification)

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **适用版本**: v3.1.0
> **前置版本**: v3.0.0 GA

> **SSOT 声明**: `gate_spec_v310.md` 是 v3.1.0 GA 门禁定义的唯一权威来源。`RELEASE_POLICY.md`、`RELEASE_LIFECYCLE.md`、`AI_COLLABORATION.md` 等文档中的门禁描述仅作引用，不得独立定义门禁检查项。

---

## 一、门禁概述

v3.1.0 GA Gate 是正式发布前的最终质量门槛，确保：
1. 所有代码层检查通过（Build/Test/Clippy/Format/Coverage/Security）
2. 所有文档层检查通过（死链/必选文档/版本一致性/用户指南）
3. 所有性能层检查通过（Point/UPDATE/DELETE QPS/TPC-H/SQL Corpus）
4. 所有稳定性测试通过（B-S1~B-S6）
5. 所有流程合规检查通过（CI/Issue/Branch）
6. 所有自我优化检查通过（TODO/Proof/GMP结构）
7. 发布前准备就绪（Release Notes/Tag）

### GA Gate 入口条件

- R-Gate (R1-R12 + R-S1~R-S6) 已通过
- Point Select QPS ≥10,000
- UPDATE QPS ≥5,000
- DELETE QPS ≥2,000
- TPC-H SF=1 22/22 无 OOM
- SQL Corpus ≥98%
- 所有已知问题已关闭或有豁免记录

---

## 二、G1-G24 检查清单（完整版）

### 2.1 代码层 Gate（G1-G6）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| G2 | Test | `cargo test --all-features` | **100% 通过** | `{passed, failed, exit_code}` |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| G4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| G5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥85% | `{total_pct, module_pcts}` |
| G6 | Security | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |

### 2.2 文档层 Gate（G7-G7d）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G7 | 死链检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| G7b | 必选文档存在性 | 检查 `docs/governance/VERSION_DOCS_SPEC.md` 定义的最小文档集 | 全部存在 | `{missing_docs}` |
| G7c | 版本号一致性 | 所有文档头部版本号为 v3.1.0 | 无遗留旧版本号 | `{mismatches}` |
| G7d | 用户指南存在性 | `docs/user/USER_MANUAL.md`, `docs/gmp-compliance/README.md` | 全部存在 | `{missing_guides}` |

### 2.3 性能层 Gate（G8-G13）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G8 | Point Select QPS | `cargo bench -- point_select` | **≥10,000 ops/s** | `{qps, threshold, pass}` |
| G9 | UPDATE QPS | `cargo bench -- update_simple` | **≥5,000 ops/s** | `{qps, threshold, pass}` |
| G10 | DELETE QPS | `cargo bench -- delete_simple` | **≥2,000 ops/s** | `{qps, threshold, pass}` |
| G11 | TPC-H SF=1 | `scripts/gate/check_tpch.sh sf=1` | 22/22 通过，无 OOM | `{passed, total, oom_count}` |
| G12 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | **≥98%** | `{passed, total, pct}` |
| G13 | Formal Proofs | `bash scripts/gate/check_proof.sh` | ≥30 proofs verified | `{count, verified_count}` |

> **注**: G8/G9/G10 必须实际运行 `cargo bench` 并解析 ops/s 输出，禁止仅依赖回归检测。

### 2.4 稳定性测试 Gate（G14）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G14-S1 | concurrency_stress_test | `cargo test --test concurrency_stress_test` | 全部通过 | `{passed, total}` |
| G14-S2 | crash_recovery_test | `cargo test --test crash_recovery_test` | 全部通过 | `{passed, total}` |
| G14-S3 | long_run_stability_test | `cargo test --test long_run_stability_test` | 全部通过 | `{passed, total}` |
| G14-S4 | wal_integration_test | `cargo test --test wal_integration_test` | 全部通过 | `{passed, total}` |
| G14-S5 | network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test` | 全部通过 | `{passed, total}` |
| G14-S6 | ssi_stress_test | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 | `{passed, total}` |

### 2.5 协议与集成测试 Gate（G15-G16）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G15 | MySQL Protocol | mysql:5.7 容器握手测试 | 连接成功 | `{handshake, query_response}` |
| G16 | B-S6 SSI Stress | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 | `{passed, total}` |

### 2.6 流程合规层 Gate（G17-G19）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G17 | CI/CD Status | Gitea API 检查 develop/v3.1.0 状态 | success 或 pending | `{status}` |
| G18 | Issue Close 验证 | 检查 milestone v3.1.0 无 OPEN issues | 无阻塞性 OPEN issue | `{open_issues, blockers}` |
| G19 | Branch Protection | Gitea API 检查分支保护 | push disabled | `{protection_enabled}` |

### 2.7 自我优化层 Gate（G20-G22）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G20 | 代码质量扫描 | `grep -r "TODO\|FIXME" crates/*/src/*.rs` | 无未解决 TODO/FIXME | `{count}` |
| G21 | Proof Registry | `docs/gmp-compliance/proof/PROOF_INDEX.md` | proof_id ≥30 | `{total, verified}` |
| G22 | GMP 文档结构 | `docs/gmp-compliance/` | proof/stability/audit/security/coverage/deployment 目录完整 | `{missing_dirs}` |

### 2.8 发布前检查 Gate（G23-G24）

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G23 | Release Notes | `docs/releases/v3.1.0/RELEASE_NOTES.md` 或 `CHANGELOG.md` | 存在 | `{exists}` |
| G24 | Tag Preparation | `git tag -l "v3.1.0"` | 未重复创建 | `{tag_exists}` |

---

## 三、覆盖率要求

### 3.1 模块级别覆盖率目标

| 模块 | A-Gate | B-Gate | R-Gate | **G-Gate** |
|------|--------|--------|--------|------------|
| executor | ≥45% | ≥60% | ≥75% | **≥80%** |
| optimizer | ≥40% | ≥50% | ≥70% | **≥70%** |
| storage | ≥15% | ≥20% | ≥40% | **≥40%** |
| catalog | ≥50% | ≥60% | ≥70% | **≥75%** |
| parser | ≥50% | ≥60% | ≥40% | **≥40%** |
| **整体** | **≥50%** | **≥75%** | **≥85%** | **≥85%** |

### 3.2 证据格式

```json
{
  "G5_coverage": {
    "total_pct": 86.5,
    "executor_pct": 81,
    "optimizer_pct": 71,
    "storage_pct": 42,
    "catalog_pct": 76,
    "parser_pct": 45,
    "pass": true
  }
}
```

---

## 四、Issue 追踪闭环要求

### 4.1 核心原则

```
门禁失败 → 必须有 Issue → 必须有修复 PR → 必须验证通过 → Issue 才能关闭
```

### 4.2 Issue 创建标准

每个门禁失败必须创建 Issue，标题格式：

| 门禁项 | Issue 标题模板 |
|--------|---------------|
| G2 测试通过率 | `[G2] 全量测试通过率 {X%}，低于 100% 要求` |
| G5 覆盖率 | `[G5] {模块} 覆盖率 {X%}，低于 {Y%} 要求` |
| G8-G10 性能 | `[G{N}] {指标} {X}，低于 {Y} 要求` |
| G11 TPC-H | `[G11] TPC-H SF=1 {N}/22 通过` |
| G14 稳定性 | `[G14-S{N}] {测试名} 未通过` |
| G12 SQL Corpus | `[G12] SQL Corpus {X%}，低于 98% 要求` |

### 4.3 Issue 关闭验证（强制）

**禁止在没有 PR 证据的情况下关闭 Issue。**

关闭前必须验证：
1. `gh issue view {id} --json closedByPullRequestsReferences` 结果非空
2. `gh pr view {pr_number} --json state,mergedAt` state=MERGED
3. 相关测试在 CI 通过
4. 门禁重新检查 PASS

---

## 五、版本延续追踪要求

### 5.1 延续触发条件

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 说明 |
|------|------|
| 修复需要 3 人周以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖 | 如 CBO 需要先完成索引选择 |

### 5.2 延续标准格式

在 `DEVELOPMENT_PLAN.md` 中建立映射：

```markdown
## v3.2.0 延续任务（来自 v3.1.0 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | v3.2.0 目标 | 验收条件 |
|----------|----------|------------|-------------|----------|
| #XXX | SQL Operations 语法支持 | 80% (44/55) | ≥95% (52/55) | test_sql_corpus_operations ≥95% |
```

---

## 六、豁免规则

以下情况可申请门禁豁免：

| 豁免类型 | 条件 | 审批人 |
|----------|------|--------|
| 覆盖率豁免 | 新增代码可证明难以测试 | Tech Lead |
| 性能豁免 | 性能测试环境不稳定 | QA Lead |
| 文档豁免 | 文档更新不影响功能 | Docs Lead |
| TPC-H 豁免 | Q17/Q18 证明是存储层限制非查询逻辑错误 | Architect |

豁免记录必须写入 `docs/governance/GATE_EXEMPTIONS.md`。

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-14 | v3.1.0 初始版本：基于 gate_spec_v300.md，整合 governance_self_improvement.md 和 gate_lifecycle_tracking.md 的完整要求 |

---

*本文档由 hermes-z6g4 维护。SSOT: gate_spec_v310.md 是 v3.1.0 GA 门禁唯一权威来源。*