# Brainstorming: 治理文档整改 — 对齐 v2.9.0 现实

> **日期**: 2026-05-05
> **问题来源**: DeepSeek 审查 `docs/governance/` 后的评估意见
> **目标**: 将治理文档从"框架设计"变为"可落地执行"

---

## Problem Statement

当前治理文档（gate_spec.md, RELEASE_POLICY.md, AI_COLLABORATION.md, GATE_CI_CD.md 等）存在三大类问题：

1. **工具不匹配**: 使用 `cargo tarpaulin`，但 v2.9.0 实际已全面切换为 `cargo-llvm-cov`
2. **阈值与实际脱节**: G-Gate 85% 覆盖率过于激进；R9 性能回归无基准无检测方法
3. **规范碎片化**: 无统一的版本文档规范，文档间相互引用但不统一

这些问题导致**规范与现实脱节**，CI 无法真正执行文档中描述的门禁流程。

---

## 5Why Deep-Dive

### Why 1: 为什么治理文档用 `cargo tarpaulin` 而不是 `cargo-llvm-cov`？

**Why 2:** 因为 gate_spec.md 初稿编写时（2026-05-01）参考了行业通用实践，但未核实 v2.9.0 实际工具链。

**Why 3:** gate_spec.md 维护者（macmini opencode）在更新时只改了版本号，未重新验证工具命令。

→ **根因**: 文档更新流程缺失"工具命令验证"步骤。

---

### Why 1: 为什么 G-Gate 要求 85% 覆盖率但无宽限条件？

**Why 2:** 85% 是行业 GA 标准，但 v2.9.0 当前 B-Gate 实际仅 84.18%，距 85% 仅差 0.82%。

**Why 3:** 治理文档设定阈值时参考了通用数据库项目惯例，未考虑 v2.9.0 作为**早期 RC 版本**的实际状态。

→ **根因**: 阈值设定未区分"长期目标"与"当前版本可达成"。

---

### Why 1: 为什么 R9 性能回归检查无法执行？

**Why 2:** gate_spec.md 只写"无性能回归"，未定义基准文件、允许浮动范围、检测脚本。

**Why 3:** v2.9.0 的性能测试基础设施（benchmark_baseline.json, check_regression.sh）尚未建立。

→ **根因**: R9 是"占位符检查项"，在基础设施未就绪时应标注为"条件性通过"或豁免。

---

### Why 1: 为什么没有统一的版本文档规范？

**Why 2:** DOCUMENT_COMPLETENESS_CHECK.md 定义了检查项，但未规定文档的元数据格式、命名规范、内容模板。

**Why 3:** 各版本文档由不同 Agent/人类维护，格式随意，导致文档审计困难。

→ **根因**: 缺少 `VERSION_DOCS_SPEC.md` 作为文档治理的"宪法"。

---

## Causal Chain

```
工具链切换 (tarpaulin → llvm-cov)
    ↓ 未更新文档
gate_spec.md R5 命令过时
    ↓ 无法在 CI 执行
CI 中的 R5 检查永远 FAIL
    ↓ 开发者绕过或忽略
门禁规范失去可信度
    ↓
整个治理框架被视为"装饰性文档"
```

---

## Root Causes (with Evidence)

### Root Cause A: gate_spec.md 中的工具命令与 v2.9.0 现实严重脱节

**证据**:
- `gate_spec.md` 第 36 行: `cargo tarpaulin ≥75%`
- `gate_spec.md` 第 228 行: `cargo tarpaulin --workspace --all-features`
- 实际 v2.9.0 CI 使用: `cargo llvm-cov --all-features --lcov --output-path lcov.info`
- 原因: cargo-tarpaulin 在 Gitea Actions 环境中编译失败，已被替换

**影响**: R5 覆盖率检查在文档里和 CI 里用的是两套完全不同的命令。

---

### Root Cause B: 覆盖率阈值仅有整体目标，缺少模块级分阶段要求

**证据**:
- gate_spec.md 第 62-70 行: A-Gate 模块覆盖率
- gate_spec.md 第 94-102 行: B-Gate 模块覆盖率（但 R-Gate 表缺失）
- DeepSeek 指出: v2.9.0 RC 报告使用 B2 executor=71.08% 作为门禁，但 gate_spec 未定义 R-Gate 模块阈值
- v2.9.0 实际: B-Gate 84.18% 整体, executor=71.08%, optimizer=63.44%, storage=15.13%

**影响**: 改进工作无明确指引，文档与实际判定结果不一致。

---

### Root Cause C: R7 文档门禁范围过窄

**证据**:
- gate_spec.md R7 仅要求 `check_docs_links.sh`（死链检查）
- 但 DeepSeek 指出: 文档完整性还需检查必选文档存在性、版本号一致性、文档与代码状态一致性
- `DOCUMENT_COMPLETENESS_CHECK.md` 已定义了更完整的检查项，但 gate_spec.md 未引用

**影响**: R7 实际只做了最低限度检查，无法发现文档缺失或版本混乱问题。

---

### Root Cause D: 门禁证据链规范缺失

**证据**:
- gate_spec.md 各检查项只写"通过标准"，不要求附证明据格式
- 实际 RC 报告中 R1-R10 状态由 Hermes Agent 主观判定，无客观证据（命令输出摘要、commit hash、产物路径）

**影响**: "CI PASS" 与 "真正验证了" 之间存在模糊地带，无法审计。

---

## Control Loop Synthesis

| 控制要素 | 当前状态 | 目标状态 |
|----------|----------|----------|
| **Sensors** | gate_spec.md 定义检查项 | 增加证据格式要求（命令输出摘要、commit、产物路径） |
| **Controllers** | 各文档自行描述门禁，无统一权威 | gate_spec.md 为唯一权威，其他文档引用它 |
| **Actuators** | CI 执行检查，但与文档不同步 | CI 脚本直接读取 gate_spec.md 中的命令，文档即执行脚本 |

---

## Proposed Approach

### Approach A: 保守修复 — 仅修正工具命令（最小改动）

**优点**: 改动最小，风险低，快速见效
**缺点**: 不解决阈值、证据链、规范碎片化问题

```
gate_spec.md:
  - tarpaulin → llvm-cov
  - 阈值不变
  - 其他文档引用 gate_spec.md
```

### Approach B: 完整整改 — 重建治理文档体系（推荐）

**优点**: 彻底解决所有根因，与 v2.9.0 现实完全对齐
**缺点**: 改动范围大，需要较多时间

核心交付物:
1. `gate_spec.md` v1.2 — 修正工具命令 + 模块阈值矩阵 + 证据格式规范
2. `VERSION_DOCS_SPEC.md` — 新增，统一版本文档标准
3. `RELEASE_POLICY.md` v2.1 — 移除重复的门禁描述，引用 gate_spec.md
4. `AI_COLLABORATION.md` v2.1 — 更新门禁描述引用
5. `GATE_CI_CD.md` v1.1 — 标注"目标架构 vs 当前实现"

**推荐 Approach B**，因为当前文档问题已严重到影响可信度，小修小补无法根治。

---

## Open Questions

- [ ] R9 性能回归检查: v2.9.0 是否已有 benchmark_baseline.json？如果没有，是延期到 v2.10.0 还是建立临时基准？
- [ ] G-Gate 85% 阈值: 是否为 v2.9.0 设置"RC 宽限条件"（关键模块达标即可，整体通过后续补丁提升）？
- [ ] R10 形式化证明: "tool_output" 字段要求是否适用于现有 18 个 proof files？是否需要追溯补充？
- [ ] `VERSION_DOCS_SPEC.md` 是否需要强制所有 v2.9.0 存量文档补充元数据（版本号、日期、阶段状态）？
- [ ] Gitea Actions runner 健康问题: ssh 到 192.168.0.252 当前失败，是否影响文档整改的提交和 CI 验证？

---

## Next Steps

1. 发布本文档到 Gitea Wiki
2. 创建 Gitea Issue，附上 Wiki 链接，邀请 DeepSeek/ChatGPT 审查
3. 收到外部 AI 审查意见后，编写 `2026-05-05-governance-doc-implementation-plan.md`
4. 通过 `subagent-driven-development` 执行整改任务
