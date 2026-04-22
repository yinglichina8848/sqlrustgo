# AGENTS.md - SQLRustGo Agent Guide

> Compact instructions for AI agents working in this repository. Based on lessons learned from past sessions.

## Branch Strategy

- **Main development branch**: `develop/v2.8.0`
- **DO NOT modify `main` branch directly**
- Create feature branches from `develop/v2.8.0`
- Use git worktrees for isolated feature work: `git worktree add .worktrees/<name> -b feature/<name>`

## Essential Commands

```bash
# Build entire project
cargo build --all-features

# Run all tests
cargo test --all-features

# Run single test
cargo test <test_name> --all-features

# Lint (required before commit)
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

Located in `scripts/gate/`:

```bash
# Check entry doc links (fast)
bash scripts/gate/check_docs_links.sh

# Check ALL doc links (slower)
bash scripts/gate/check_docs_links.sh --all

# Coverage check
bash scripts/gate/check_coverage.sh

# Security check
bash scripts/gate/check_security.sh
```

## Architecture

```
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                 │  ← Query execution
├─────────────────────────────────────┤
│           parser/                    │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           storage/                   │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                 │  ← WAL, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP server/client
├─────────────────────────────────────┤
│           types/                     │  ← Value, SqlError
└─────────────────────────────────────┘
```

## Crates (Workspace Members)

Key crates in `crates/`:
- `parser`, `planner`, `optimizer`, `executor` - Query processing
- `storage` - Buffer pool, file storage, B+ tree, columnar storage
- `transaction` - WAL, MVCC, transaction manager
- `network` - TCP server with MySQL-style protocol
- `vector`, `graph` - Advanced storage (vector index, graph store)
- `catalog`, `types` - Schema and type system

## Important Constraints

1. **Clippy must pass**: `cargo clippy --all-features -- -D warnings` (no warnings allowed)
2. **Format must pass**: `cargo fmt --check --all`
3. **Doc links must be valid**: Run `check_docs_links.sh` before committing doc changes
4. **Test memory limit**: 8GB per test (configured in Cargo.toml)
5. **Rust edition**: 2021 with Tokio async runtime

## Common Pitfalls

| Issue | Prevention |
|-------|------------|
| Broken doc links | Always run `scripts/gate/check_docs_links.sh` after modifying markdown |
| Missing workspace deps | Use `-p <package>` flag for single crate operations |
| Slow builds | Use `cargo check` for fast compilation checks |
| Missing features | Use `--all-features` to enable all feature flags |

## Existing Instruction Files

- `.claude/CLAUDE.md` - Claude Code specific guidance
- `AGENT.md` - Issue-specific guide (DiskGraphStore implementation)
- `ARCHITECTURE_RULES.md` - Architecture decisions
- `BRANCH_GOVERNANCE.md` - Branch and release workflow
- `docs/governance/ISSUE_CLOSING_VERIFICATION.md` - **Issue 关闭验证流程 (强制执行)**

## Issue 关闭规则 (强制)

**禁止手动关闭没有 PR 合并的 Issue。**

关闭 Issue 前必须验证：
```bash
# 1. 检查是否有 PR 关闭该 Issue
gh issue view <id> --json closedByPullRequestsReferences

# 结果非空 → 可以关闭
# 结果为空 → 禁止手动关闭，除非任务取消
```

详见: `docs/governance/ISSUE_CLOSING_VERIFICATION.md`

## Test Execution Notes

- Integration tests are in `tests/` directory
- E2E tests in `tests/e2e/`
- Crate-specific tests in each crate's `tests/` or `src/`
- Use `--test <test_name>` to run specific test files
