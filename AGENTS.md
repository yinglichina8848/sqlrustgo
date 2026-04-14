# SQLRustGo AGENTS.md

**Generated**: 2026-04-14
**Version**: v1.6.1
**Branch**: `feature/foreign-key-table-constraint`

---

## OVERVIEW

SQLRustGo 是 Rust 实现的关系型数据库教学项目，支持 SQL-92 子集。采用现代分层架构：Parser → Planner → Optimizer → Executor → Storage。

---

## PROJECT SCALE

| Metric | Value |
|--------|-------|
| Crates | 37 |
| Code Files | ~2000 |
| Max Depth | 5 |
| Large Files (>500loc) | ~50 |

---

## WORKSPACE STRUCTURE

```
sqlrustgo/
├── crates/           # 37 个 workspace member
│   ├── parser/       # SQL → AST
│   ├── planner/      # AST → Logical Plan
│   ├── optimizer/    # Cascades CBO
│   ├── executor/     # Query Execution
│   ├── storage/      # BufferPool, B+Tree, WAL
│   ├── transaction/  # TxManager
│   ├── network/      # MySQL Protocol
│   ├── server/       # TCP Server
│   ├── graph/        # Graph Module (NEW)
│   └── ...           # 28 more crates
├── tests/            # Integration tests
├── benches/          # Benchmarks
├── docs/             # Documentation
└── scripts/          # Build scripts
```

---

## KEY CONVENTIONS

### Rust Edition & Toolchain
- **Edition**: 2021
- **MSRV**: Rust 1.85+
- **Async**: Tokio runtime

### Build Commands
```bash
cargo build --all-features    # Full build
cargo test --all-features     # All tests
cargo clippy -- -D warnings    # Strict lint
cargo fmt --check             # Format check
```

### Commit Style
```
<type>(<scope>): <subject>

Types: feat, fix, test, docs, refactor, chore
Scope: parser, executor, storage, etc.
```

---

## ACTIVE ISSUES

### Issue #1379: FOREIGN KEY 约束 (当前分支)

**目标**: 实现表级 FOREIGN KEY 约束语法

| Component | Status | Notes |
|-----------|--------|-------|
| Parser | ✅ Done | TableConstraint parsing works |
| Executor | ❌ Todo | FK enforcement not implemented |
| Tests | ⚠️ Partial | Compile but fail at runtime |

**Files Changed**:
- `crates/parser/src/parser.rs`
- `crates/parser/src/token.rs`
- `crates/parser/src/lexer.rs`
- `crates/parser/src/lib.rs`
- `tests/integration/foreign_key_table_constraint_test.rs` (new)

**PR**: https://github.com/minzuuniversity/sqlrustgo/pull/1427

### Issue #1378: Graph 持久化 (SOUL.md)

**目标**: 实现 DiskGraphStore 支持 WAL 崩溃恢复

**Status**: Design phase

---

## CODE MAP (Key Symbols)

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `CreateTableStatement` | struct | parser | CREATE TABLE AST |
| `TableConstraint` | enum | parser | FK/PK/UNIQUE constraints |
| `ExecutionEngine` | struct | executor | Query execution |
| `MemoryStorage` | struct | storage | In-memory storage engine |
| `WalManager` | struct | transaction | Write-ahead logging |

---

## THIS PROJECT'S RULES

### Must Use
- `cargo clippy -- -D warnings` before commit
- `cargo fmt` before commit
- Workspace member crates for new modules

### Must NOT
- `as any` type suppression
- Empty catch blocks `catch(e) {}`
- Commit without running tests

### Project-Specific
- All public API documented in `lib.rs`
- Integration tests in `tests/integration/`
- Unit tests co-located with source

---

## COMMON TASKS

```bash
# Run parser tests
cargo test -p sqlrustgo-parser

# Run integration tests
cargo test --test '*'

# Run single test
cargo test test_name --all-features

# Add new crate
# 1. Create in crates/<name>/src/lib.rs
# 2. Add to Cargo.toml workspace.members
# 3. Add to .github/workflows/ci.yml

# Build release
cargo build --release --all-features
```

---

## GOTCHAS

1. **Parser returns `Statement` not `CreateTableStatement`** - Need `.try_into()` cast
2. **`Arc<RwLock<T>>` wrapper** - Storage is thread-safe via this pattern
3. **WAL sync** - Must call `wal.sync()` after write operations
4. **Token stream is consumed** - Parser takes ownership, no rewind

---

## RELEVANT DOCS

| File | Purpose |
|------|---------|
| `.claude/CLAUDE.md` | Claude Code 指导 |
| `SOUL.md` | Issue #1379 进度记录 |
| `AGENT.md` | Issue #1378 执行指南 |
| `docs/architecture/` | 架构文档 |
| `docs/releases/v1.6.0/` | Release notes |

---

**Agent 使用此文件时请遵循 SQLRustGo 项目规范。**
