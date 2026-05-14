# AGENTS.md - SQLRustGo Agent Guide

> **版本**: 2.0
> **更新日期**: 2026-05-14
> **维护人**: hermes-z6g4

> **强制执行声明**: 本文档是 AI 执行 SQLRustGo 相关任务的强制规则。AI 必须遵循本文档的所有规定，以及 docs/governance/ 下的 SSOT 文档。

---

## 一、执行保证机制

### 1.1 规则层级（从高到低）

```
1. AGENTS.md（最高优先级）
   ↓
2. docs/governance/GOVERNANCE_INDEX.md（导航索引）
   ↓
3. docs/governance/GATE_SPEC_MASTER.md（门禁规范 SSOT）
   ↓
4. docs/governance/DEVELOPMENT_PLAN_TEMPLATE.md（开发计划模版）
   ↓
5. docs/governance/TEST_PLAN_TEMPLATE.md（测试计划模版）
   ↓
6. docs/governance/GATE_CHECKLIST_TEMPLATE.md（门禁清单模版）
   ↓
7. scripts/gate/check_{phase}_v{VERSION}.sh（门禁脚本）
```

### 1.2 任务分类与强制规则

| 任务类型 | 必须遵循 | 执行前必读 |
|----------|----------|------------|
| 版本开发计划 | DEVELOPMENT_PLAN_TEMPLATE.md | GOVERNANCE_INDEX.md |
| 测试计划 | TEST_PLAN_TEMPLATE.md | GOVERNANCE_INDEX.md |
| 门禁检查 | GATE_CHECKLIST_TEMPLATE.md | GATE_SPEC_MASTER.md |
| 问题修复 | 本文档 + 相关模版 | GOVERNANCE_INDEX.md |

---

## 二、Communication Principle

**必须使用中文沟通。始终使用中文回应用户，除非用户明确要求使用其他语言。**

---

## 三、Governance 任务执行规则

### 3.1 任务识别规则

```
当用户要求执行以下任务时，AI 必须使用对应的模版：
```

| 用户要求关键词 | 识别为 | 必须使用的模版 |
|----------------|--------|----------------|
| "开发计划"、"版本计划"、"版本开发" | 版本开发计划 | DEVELOPMENT_PLAN_TEMPLATE.md |
| "测试计划"、"测试用例"、"测试策略" | 测试计划 | TEST_PLAN_TEMPLATE.md |
| "门禁检查"、"Gate"、"门禁报告" | 门禁检查 | GATE_CHECKLIST_TEMPLATE.md |
| "Alpha Gate"、"Beta Gate"、"RC Gate"、"GA Gate" | 门禁检查 | GATE_SPEC_MASTER.md + GATE_CHECKLIST_TEMPLATE.md |
| "执行门禁"、"跑门禁" | 执行门禁 | 执行 scripts/gate/check_{phase}_v{VERSION}.sh |

### 3.2 强制模版使用

```
❌ 禁止: 自由格式创建开发计划/测试计划/门禁清单
✅ 必须: 基于对应模版创建
✅ 必须: 包含模版要求的所有章节
✅ 必须: 使用模版指定的文件路径
```

### 3.3 门禁检查执行规则

```
步骤 1: 识别门禁阶段
    ↓
步骤 2: 执行 scripts/gate/check_{phase}_v{VERSION}.sh
    ↓
步骤 3: 记录所有检查结果（PASS/FAIL/SKIP）
    ↓
步骤 4: FAIL 项 → 创建 Issue → 修复 PR → 验证
    ↓
步骤 5: 生成 GATE_CHECKLIST_TEMPLATE.md 格式的报告
    ↓
步骤 6: 发布 Issue 记录门禁状态
```

### 3.4 版本开发计划执行规则

```
步骤 1: 阅读 GOVERNANCE_INDEX.md
    ↓
步骤 2: 使用 DEVELOPMENT_PLAN_TEMPLATE.md 创建计划
    ↓
步骤 3: 创建 docs/releases/v{VERSION}/DEVELOPMENT_PLAN.md
    ↓
步骤 4: 包含所有 9 个必需章节
    ↓
步骤 5: Issue 引用必须有效
    ↓
步骤 6: 版本延续任务映射到上版本未完成任务
```

---

## 四、Issue 追踪强制规则

### 4.1 Issue 创建触发条件

```
门禁 FAIL → 必须创建 Issue
    ↓
Issue 标题格式: [{GATE_ITEM}] {简短描述}
    ↓
Issue 必须包含:
  - 门禁来源 (脚本路径、检查项编号)
  - 检查命令
  - 失败输出摘要
  - 根因分析
  - 影响范围
  - 验收条件
```

### 4.2 Issue 关闭强制验证

```
禁止: 没有 PR 证据就关闭 Issue
    ↓
关闭前必须验证:
  1. gh issue view {id} --json closedByPullRequestsReferences (结果非空)
  2. gh pr view {pr_number} --json state,mergedAt (state=MERGED)
  3. 相关测试在 CI 通过
  4. 门禁重新检查 PASS
```

---

## 五、Branch Strategy

- **Main development branch**: `develop/v3.1.0` (Beta)
- **DO NOT modify `main` branch directly**
- Create feature branches from `develop/v3.1.0`
- Use git worktrees for isolated feature work: `git worktree add .worktrees/<name> -b feature/<name>`

---

## 六、Essential Commands

```bash
# Build (use --all-features to enable all feature flags)
cargo build --all-features

# Fast compilation check (no linking)
cargo check --all-features

# Run all tests
cargo test --all-features

# Run single test
cargo test <test_name> --all-features

# Run tests in specific crate
cargo test -p sqlrustgo-executor --all-features

# Lint (required before commit - zero warnings required)
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt --check --all

# Format fix
cargo fmt --all

# Run REPL
cargo run --bin sqlrustgo

# Run doc tests
cargo test --doc
```

---

## 七、Gate/Validation Scripts

```bash
# Doc links check (fast)
bash scripts/gate/check_docs_links.sh

# Full doc links check
bash scripts/gate/check_docs_links.sh --all

# Coverage check (v3.1.0)
bash scripts/gate/check_coverage.sh

# Security check
bash scripts/gate/check_security.sh

# L0 smoke test
bash scripts/gate/check_l0_smoke.sh
```

---

## 八、Architecture

```
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                  │  ← Query execution
│           parser/                   │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           planner/                   │  ← AST → Logical plan
│           optimizer/                 │  ← Query optimization (CBO)
├─────────────────────────────────────┤
│           storage/                  │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                │  ← WAL, MVCC, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP server/MySQL protocol
├─────────────────────────────────────┤
│           types/                    │  ← Value, SqlError
└─────────────────────────────────────┘
```

---

## 九、Crates (Workspace Members)

Key crates in `crates/`:
- `parser`, `planner`, `optimizer`, `executor` - Query processing
- `storage` - Buffer pool, file storage, B+ tree, columnar storage
- `transaction` - WAL, MVCC (Snapshot Isolation), transaction manager
- `network` - TCP server with MySQL-style protocol
- `vector`, `graph` - Advanced storage (vector index, graph store)
- `catalog`, `types` - Schema and type system
- `server` - Database server entry point

---

## 十、重要约束

1. **Clippy must pass**: `cargo clippy --all-features -- -D warnings` (zero warnings allowed)
2. **Format must pass**: `cargo fmt --check --all`
3. **Doc links must be valid**: Run `check_docs_links.sh` after modifying markdown
4. **Test memory limit**: 8GB per test (configured in Cargo.toml)
5. **Rust edition**: 2021 with Tokio async runtime
6. **Workspace packages**: Use `-p <package>` flag for single crate operations

---

## 十一、GMP 文档体系

### 外部知识库

| 资源 | 地址 | 用途 |
|------|------|------|
| **LLM-Wiki** | `ssh://git@192.168.0.252:222/openclaw/sqlrustgo-wiki.git` | 项目文档、架构决策记录 |
| **GBrain** | `ssh://git@192.168.0.252:222/openclaw/ai-brain.git` | 可检索知识图谱、规则、模式库 |

### 本地知识库结构

```
~/wiki/gbrain/sqlrustgo/    # GBrain 本地克隆（从 ai-brain.git 克隆）
├── rules/                   # 治理规则、编码规范
├── patterns/                # 重复问题模式（如 Gitea Actions Cache 卡死）
├── architecture/            # 架构决策记录 (ADR)
└── decisions/               # 具体决策备忘

docs/wiki/                   # QMD Wiki（结构化流程文档）
```

---

## 十二、Harness Engineering 体系

### Gate 脚本 (`scripts/gate/`)

v3.1.0 门禁脚本：

| 脚本 | 用途 |
|------|------|
| `check_alpha_v310.sh` | v3.1.0 Alpha Gate |
| `check_beta_v310.sh` | v3.1.0 Beta Gate |
| `check_rc_v310.sh` | v3.1.0 RC Gate |
| `check_ga_v310.sh` | v3.1.0 GA Gate |
| `check_coverage.sh` | 覆盖率检查 (≥75%/85%) |
| `check_security.sh` | 安全扫描 |
| `check_sql_compat.sh` | SQL 兼容性检查 |
| `check_l0_smoke.sh` | L0 smoke 测试 |

### Beta Gate 要求 (v3.1.0 B1-B9)

| 检查项 | 要求 |
|--------|------|
| B1 Build | 编译通过 |
| B2 Test ≥90% | L1 core crates 通过率 ≥90% |
| B3 Clippy | 零警告 |
| B4 Format | `cargo fmt --check` 通过 |
| B5 Coverage ≥75% | 覆盖率 ≥75% (Beta) / 85% (GA) |
| B6 Security | 无已知漏洞 |
| B7 SQL Compat | SQL Corpus ≥80% |
| B8 TPC-H SF=1 | TPC-H 基准可运行 |
| B9 Proof | 证明文件存在 |

---

## 十三、多平台协作架构

### 三平台角色

| 平台 | 角色 | 主要职责 |
|------|------|---------|
| **Mac Mini (Brain)** | Orchestrator | 任务编排、PR 合并、知识管理 |
| **Z6G4** | Heavy Worker | 大规模编译、测试、性能基准 |
| **ai@250** | Light Worker | 辅助编译、文档生成 |

### Git 身份要求

允许的 Email（pre-commit hook 强制检查）：
- `openheart@gaoyuanyiyao.com`
- `hermes-macmini@gaoyuanyiyao.com`
- `hermes-z6g4@gaoyuanyiyao.com`
- `ci@example.com` (CI Bot)

### 分支保护规则

- `develop/v3.1.0`: 禁止直接推送，必须通过 PR
- 使用 `force_merge=true` 绕过保护合并（需 API）

---

## 十四、Governance 执行检查清单

AI 在执行 governance 相关任务前，必须检查：

```
□ 是否识别到 governance 相关任务？
□ 是否阅读了 GOVERNANCE_INDEX.md？
□ 是否使用了正确的模版？
□ 模版章节是否完整？
□ 文件路径是否正确？
□ Issue 追踪是否遵循规则？
□ 门禁检查结果是否记录？
```

---

## 十五、GitNexus Code Intelligence

<!-- gitnexus:start -->
This project is indexed by GitNexus as **sqlrustgo** (64373 symbols, 95746 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

### Always Do
- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

### Never Do
- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

### Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/sqlrustgo/context` | Codebase overview, check index freshness |
| `gitnexus://repo/sqlrustgo/clusters` | All functional areas |
| `gitnexus://repo/sqlrustgo/processes` | All execution flows |
| `gitnexus://repo/sqlrustgo/process/{name}` | Step-by-step execution trace |
<!-- gitnexus:end -->

---

*本文档由 hermes-z6g4 维护。版本 2.0 新增 Governance 执行规则。*