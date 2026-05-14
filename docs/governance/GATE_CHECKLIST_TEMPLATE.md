# GATE_CHECKLIST_TEMPLATE.md — SQLRustGo 版本门禁检查清单模版

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **用途**: 所有版本的门禁检查清单必须基于本文档创建
> **SSOT**: 本模版的使用是强制性的，不使用本模版创建的门禁检查清单视为不合格

---

## 一、模版使用说明

### 1.1 强制使用声明

```
❌ 不允许: 自由格式的门禁检查
✅ 必须: 基于本文档模版创建
✅ 必须: 在进入下一阶段前完成所有检查
✅ 必须: 所有失败项必须有对应的 Issue/PR
```

### 1.2 门禁检查清单文件命名

```
docs/releases/v{VERSION}/{PHASE}_GATE_CHECKLIST.md
scripts/gate/check_{phase}_v{VERSION}.sh
```

示例：
- `docs/releases/v3.1.0/BETA_GATE_CHECKLIST.md`
- `scripts/gate/check_beta_v310.sh`

### 1.3 检查类型

| 类型 | 说明 | 执行时机 |
|------|------|----------|
| **Pre-Gate** | 进入门禁前的自检 | 每次门禁前 |
| **Gate** | 正式门禁检查 | 阶段转换时 |
| **Post-Gate** | 门禁通过后的收尾 | 阶段转换后 |

### 1.4 检查结果分类

| 结果 | 说明 | 处理方式 |
|------|------|----------|
| PASS | 完全满足 | 进入下一阶段 |
| FAIL | 不满足 | 必须修复或申请豁免 |
| SKIP | 条件不满足 | 需要人工判断 |
| N/A | 不适用 | 记录原因 |

---

## 二、标准章节内容

### 第一章: 门禁信息

```markdown
# {VERSION} {PHASE} Gate Checklist

> **版本**: {VERSION}-{PHASE}-gate
> **创建日期**: {YYYY-MM-DD}
> **维护人**: {name}
> **阶段**: {Alpha/Beta/RC/GA}
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

## 门禁信息

### 1.1 门禁定义

| 属性 | 值 |
|------|-----|
| 门禁类型 | {A/B/R/G}-Gate |
| 执行日期 | {YYYY-MM-DD} |
| 执行人 | {name} |
| 脚本 | `scripts/gate/check_{phase}_v{VERSION}.sh` |
| 规范版本 | gate_spec_v{VERSION}.md |

### 1.2 入口条件

- [ ] {入口条件1}
- [ ] {入口条件2}
- [ ] {入口条件3}

### 1.3 通过标准

```
门禁通过 = (所有 MANDATORY 项 PASS) + (所有 FAIL 项有 Issue/PR) + (所有豁免项已审批)
```

---

## 三、Pre-Gate 自检清单

> 在执行正式门禁前，先进行自检

### 3.1 代码准备

| 检查项 | 方法 | 结果 | 说明 |
|--------|------|------|------|
| 代码已提交 | `git log --oneline -1` | {SHA} | 最新 commit |
| 代码已推送 | `git push --dry-run` | {status} | 无未推送 commit |
| 分支正确 | `git branch` | {branch} | 应为 develop/v{VERSION} |
| WORKSPACE 干净 | `git status` | {status} | 无未提交更改 |

### 3.2 环境准备

| 检查项 | 方法 | 结果 | 说明 |
|--------|------|------|------|
| Rust 版本 | `rustc --version` | {version} | {要求版本} |
| Cargo 版本 | `cargo --version` | {version} | {要求版本} |
| llvm-cov | `cargo llvm-cov --version` | {version} | {要求版本} |
| cargo-audit | `cargo audit --version` | {version} | {要求版本} |

### 3.3 数据准备

| 检查项 | 方法 | 结果 | 说明 |
|--------|------|------|------|
| TPC-H 数据 SF=0.1 | `ls tpch-data/sf0.1/` | {files} | {要求} |
| TPC-H 数据 SF=1 | `ls tpch-data/sf1/` | {files} | {要求} |
| SQL Corpus | `ls sql-corpus/` | {count} | {要求数量} |
```

---

## 四、正式门禁检查

### 4.1 代码质量门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G1 | Build | `cargo build --release --workspace` | 无错误 | {PASS/FAIL} | {evidence} | #{issue} |
| G2 | Test | `cargo test --all-features` | {threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | {PASS/FAIL} | {evidence} | #{issue} |
| G4 | Format | `cargo fmt --all -- --check` | 无格式错误 | {PASS/FAIL} | {evidence} | #{issue} |
| G5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥{threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G6 | Security | `cargo audit` | 无高危漏洞 | {PASS/FAIL} | {evidence} | #{issue} |

### 4.2 功能门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G7 | TPC-H SF={SF} | `check_tpch.sh` | {threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G8 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥{threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G9 | CBO Tests | `cargo test --test cbo_integration_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
```

### 4.3 性能门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G10 | Point Select QPS | `cargo bench -- point_select` | ≥{threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G11 | UPDATE QPS | `cargo bench -- update_simple` | ≥{threshold} | {PASS/FAIL} | {evidence} | #{issue} |
| G12 | DELETE QPS | `cargo bench -- delete_simple` | ≥{threshold} | {PASS/FAIL} | {evidence} | #{issue} |
```

### 4.4 稳定性门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G-S1 | concurrency_stress | `cargo test --test concurrency_stress_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
| G-S2 | crash_recovery | `cargo test --test crash_recovery_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
| G-S3 | long_run_stability | `cargo test --test long_run_stability_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
| G-S4 | wal_integration | `cargo test --test wal_integration_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
| G-S5 | network_tcp_smoke | `cargo test --test network_tcp_smoke_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
| G-S6 | ssi_stress | `cargo test --test ssi_stress_test` | 全部通过 | {PASS/FAIL} | {evidence} | #{issue} |
```

### 4.5 文档门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G-D1 | 死链检查 | `check_docs_links.sh` | 无死链 | {PASS/FAIL} | {evidence} | #{issue} |
| G-D2 | 必选文档 | 检查 `VERSION_DOCS_SPEC.md` | 全部存在 | {PASS/FAIL} | {evidence} | #{issue} |
| G-D3 | 版本一致性 | 文档头部版本号 | v{VERSION} | {PASS/FAIL} | {evidence} | #{issue} |
| G-D4 | 用户指南 | `docs/user/USER_MANUAL.md` | 存在 | {PASS/FAIL} | {evidence} | #{issue} |
```

### 4.6 流程合规门禁

| # | 检查项 | 方法 | 通过标准 | 结果 | 证据 | Issue |
|---|--------|------|----------|------|------|-------|
| G-P1 | CI Status | Gitea API | success/pending | {PASS/FAIL} | {evidence} | #{issue} |
| G-P2 | Issue 关闭 | API 查询 | 无 OPEN blocker | {PASS/FAIL} | {evidence} | #{issue} |
| G-P3 | 分支保护 | API 查询 | push disabled | {PASS/FAIL} | {evidence} | #{issue} |
```

---

## 五、检查结果汇总

### 5.1 汇总表

| 类别 | 总数 | PASS | FAIL | SKIP | N/A |
|------|------|------|------|------|-----|
| 代码质量 | {N} | {N} | {N} | {N} | {N} |
| 功能 | {N} | {N} | {N} | {N} | {N} |
| 性能 | {N} | {N} | {N} | {N} | {N} |
| 稳定性 | {N} | {N} | {N} | {N} | {N} |
| 文档 | {N} | {N} | {N} | {N} | {N} |
| 流程合规 | {N} | {N} | {N} | {N} | {N} |
| **总计** | **{N}** | **{N}** | **{N}** | **{N}** | **{N}** |

### 5.2 门禁结果

```
╔════════════════════════════════════════════════════════════╗
║  {PHASE}-Gate 结果                                         ║
╠════════════════════════════════════════════════════════════╣
║  PASS: {N}                                                 ║
║  FAIL: {N}                                                 ║
║  SKIP: {N}                                                 ║
║  结果: {PASS/FAIL}                                         ║
╚════════════════════════════════════════════════════════════╝
```

---

## 六、失败项处理

### 6.1 失败项清单

| # | 检查项 | 失败原因 | Issue | PR | 状态 |
|---|--------|----------|-------|-----|-------|
| {N} | {item} | {reason} | #{issue} | #{pr} | {FIXED/IN_PROGRESS/EXEMPTED} |

### 6.2 Issue 追踪

```markdown
## 门禁失败 Issue 模板

### Issue 标题
[{GATE_ITEM}] {简短描述}

### Issue 内容
- 门禁脚本: `check_{phase}_v{VERSION}.sh`
- 检查项: G{N}
- 命令: `{实际执行的命令}`
- 失败输出:
\`\`\`
{paste output}
\`\`\`

### 根因分析
{analysis}

### 影响范围
- 阻塞: {PHASE}-Gate
- 影响: {other impacts}

### 验收条件
- [ ] {condition 1}
- [ ] {condition 2}

### 追踪信息
- milestone: v{VERSION}-{PHASE}
- labels: source/gate-{phase}, type/{type}
```

### 6.3 豁免申请

| 检查项 | 豁免原因 | 审批人 | 复审日期 | 豁免 ID |
|--------|----------|--------|----------|---------|
| {item} | {reason} | {approver} | {date} | EX-{N} |

---

## 七、Post-Gate 收尾

### 7.1 门禁通过后操作

| 操作 | 执行人 | 日期 | 状态 |
|------|--------|------|------|
| 更新 milestone 状态 | {name} | {date} | {DONE} |
| 创建下一阶段分支 | {name} | {date} | {DONE} |
| 通知团队 | {name} | {date} | {DONE} |
| 更新开发计划 | {name} | {date} | {DONE} |

### 7.2 门禁报告

```json
{
  "gate": "{PHASE}-GATE-v{VERSION}",
  "commit": "{sha}",
  "date": "{YYYY-MM-DD}",
  "executor": "{name}",
  "status": "PASS|FAIL",
  "summary": {
    "total": {N},
    "passed": {N},
    "failed": {N},
    "skipped": {N}
  },
  "evidence": {
    "build": { "exit_code": {N} },
    "test": { "passed": {N}, "failed": {N} },
    "clippy": { "warnings": {N} },
    "coverage": { "total_pct": {N} }
  },
  "blockers": [
    {
      "item": "{item}",
      "issue": "#{issue}",
      "pr": "#{pr}",
      "status": "{FIXED|IN_PROGRESS|EXEMPTED}"
    }
  ]
}
```

---

## 八、审查与签名

### 8.1 执行人确认

| 角色 | 姓名 | 日期 | 签名 |
|------|------|------|------|
| 执行人 | {name} | {date} | {signature} |
| 审查人 | {name} | {date} | {signature} |
| 批准人 | {name} | {date} | {signature} |

### 8.2 版本回顾

```markdown
## {VERSION} {PHASE}-Gate 回顾

### 做得好的
- {what went well}

### 需要改进的
- {what needs improvement}

### 下次类似工作的建议
- {suggestions for future similar work}
```

---

## 九、附录

### 9.1 检查脚本

```bash
# 执行门禁检查
bash scripts/gate/check_{phase}_v{VERSION}.sh

# 输出到文件
bash scripts/gate/check_{phase}_v{VERSION}.sh 2>&1 | tee gate-output.txt
```

### 9.2 快速参考

| 命令 | 用途 |
|------|------|
| `cargo build --release --workspace` | Release 构建 |
| `cargo test --all-features` | 全量测试 |
| `cargo clippy --all-features -- -D warnings` | Lint 检查 |
| `cargo fmt --all -- --check` | 格式检查 |
| `cargo llvm-cov --all-features --lcov` | 覆盖率 |
| `cargo audit` | 安全审计 |
| `bash scripts/gate/check_docs_links.sh` | 文档链接 |

---

## 十、检查清单使用规则

### 10.1 强制规则

1. **每个门禁周期必须使用此清单**
2. **所有检查项必须记录结果**
3. **所有 FAIL 项必须创建 Issue**
4. **Issue 必须关联到对应的 PR**
5. **PR 合并后才能关闭 Issue**
6. **禁止在没有 PR 证据的情况下关闭 Issue**

### 10.2 禁止的模式

```
❌ 门禁 FAIL → 跳过 → 合并代码 → 问题丢失
❌ Issue 已创建 → 未关联 PR → 无人追踪
❌ 检查通过 → 未记录证据 → 后续无法复现
❌ 豁免未申请 → 直接忽略 → 违反流程
```

### 10.3 正确的模式

```
✅ 门禁检查 → 记录结果 → FAIL → 创建 Issue → 修复 PR → 验证 PASS → 关闭 Issue
✅ 门禁检查 → 记录结果 → FAIL → 评估豁免 → 申请审批 → 记录到 GATE_EXEMPTIONS.md
✅ 门禁通过 → 记录证据 → 发布报告 → 更新 milestone → 通知团队
```

---

## 十一、模版变更记录

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本 | hermes-z6g4 |

---

*本文档是 SQLRustGo 版本门禁检查清单的标准模版。不使用本模版的门禁检查清单视为不合格。*