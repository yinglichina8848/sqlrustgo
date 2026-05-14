# AGENTS.md - SQLRustGo Agent Guide

> Compact instructions for AI agents. Based on lessons learned from past sessions.

## Communication Principle

**必须使用中文沟通。始终使用中文回应用户，除非用户明确要求使用其他语言。**

## Branch Strategy

- **Main development branch**: `develop/v3.1.0` (Beta)
- **DO NOT modify `main` branch directly**
- Create feature branches from `develop/v3.1.0`
- Use git worktrees for isolated feature work: `git worktree add .worktrees/<name> -b feature/<name>`

## Essential Commands

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

## Gate/Validation Scripts

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

## Architecture

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

## Crates (Workspace Members)

Key crates in `crates/`:
- `parser`, `planner`, `optimizer`, `executor` - Query processing
- `storage` - Buffer pool, file storage, B+ tree, columnar storage
- `transaction` - WAL, MVCC (Snapshot Isolation), transaction manager
- `network` - TCP server with MySQL-style protocol
- `vector`, `graph` - Advanced storage (vector index, graph store)
- `catalog`, `types` - Schema and type system
- `server` - Database server entry point

## Important Constraints

1. **Clippy must pass**: `cargo clippy --all-features -- -D warnings` (zero warnings allowed)
2. **Format must pass**: `cargo fmt --check --all`
3. **Doc links must be valid**: Run `check_docs_links.sh` before committing doc changes
4. **Test memory limit**: 8GB per test (configured in Cargo.toml)
5. **Rust edition**: 2021 with Tokio async runtime
6. **Workspace packages**: Use `-p <package>` flag for single crate operations

## Common Pitfalls

| Issue | Prevention |
|-------|------------|
| Broken doc links | Run `scripts/gate/check_docs_links.sh` after modifying markdown |
| Missing workspace deps | Use `-p <package>` flag for single crate operations |
| Slow builds | Use `cargo check --all-features` for fast compilation checks |
| Missing features | Use `--all-features` flag for all cargo commands |
| Wrong branch | Branch is `develop/v3.0.0`, not `develop/v2.9.0` |

## Existing Instruction Files

- `.claude/CLAUDE.md` - Claude Code specific guidance (includes GitNexus)
- `ARCHITECTURE_RULES.md` - Architecture decisions
- `BRANCH_GOVERNANCE.md` - Branch and release workflow
- `docs/governance/ISSUE_CLOSING_VERIFICATION.md` - Issue closing verification

## Issue 关闭规则

**禁止手动关闭没有 PR 合并的 Issue。**

关闭 Issue 前必须验证：
```bash
gh issue view <id> --json closedByPullRequestsReferences
# 结果非空 → 可以关闭
# 结果为空 → 禁止手动关闭
```

## Git Remote

**Primary remote: Gitea** — `http://192.168.0.252:3000/openclaw/sqlrustgo.git`

| Remote | Purpose |
|--------|---------|
| origin | Gitea (primary) |
| github | GitHub mirror (read-only) |
| gitee | Gitee mirror (read-only) |
| gitcode | GitCode mirror (read-only) |

## Testing Notes

- Integration tests in `tests/` directory
- E2E tests in `tests/e2e/`
- Crate-specific tests in each crate's `tests/` or `src/`
- Use `--test <test_name>` to run specific test files
- Test memory limit: 8GB (configured in `Cargo.toml`)

## Version

Current: **v3.1.0 Beta** (Compact Row Format)
- Version file: `VERSION`
- Current branch: `develop/v3.1.0`
- Previous stable: v3.0.0 (GA)

**v3.1.0 Gate Scripts**:
```bash
bash scripts/gate/check_alpha_v310.sh   # Alpha gate
bash scripts/gate/check_beta_v310.sh    # Beta gate
bash scripts/gate/check_rc_v310.sh      # RC gate
bash scripts/gate/check_ga_v310.sh      # GA gate
```

---

## LLM-Wiki 与 GBrain 知识管理

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

### 知识沉淀流程

每次完成复杂任务后：
1. 首次解决某类问题 → 创建 GBrain Pattern
2. 学到项目规则 → 更新 Hermes Memory
3. 流程改进 → 更新 ADR
4. 对外接口变更 → 更新 GitHub Wiki

---

## Harness Engineering 体系

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

### CI/CD 基础设施

- **Runner**: Nomad + Gitea Actions
- **Runner 标签**: `hp-z6g4`, `z440`
- **SSH 用户**: `openclaw`（不是 `admin`）
- **验证命令**: `ssh openclaw@192.168.0.252 "nomad node status"`

---

## 多平台协作架构

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

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **sqlrustgo** (64373 symbols, 95746 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/sqlrustgo/context` | Codebase overview, check index freshness |
| `gitnexus://repo/sqlrustgo/clusters` | All functional areas |
| `gitnexus://repo/sqlrustgo/processes` | All execution flows |
| `gitnexus://repo/sqlrustgo/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
