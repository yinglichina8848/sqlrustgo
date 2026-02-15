# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SQLRustGo is a Rust implementation of a relational database system supporting SQL-92 subset. Built from scratch with a modern layered architecture.

## Common Commands

```bash
# Build
cargo build --all-features

# Run tests
cargo test --all-features

# Run a single test
cargo test test_name --all-features

# Lint with clippy
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt --check --all

# Doc tests
cargo test --doc

# Run REPL
cargo run --bin sqlrustgo
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

## Key Modules

| Module | Purpose |
|--------|---------|
| `lexer` | Tokenizes SQL input |
| `parser` | Parses tokens into Statement AST |
| `storage` | Page management, BufferPool (LRU), B+ Tree indexing |
| `executor` | Executes SQL statements |
| `transaction` | Write-Ahead Log, BEGIN/COMMIT/ROLLBACK |
| `network` | TCP server/client with MySQL-style protocol |

## Rust Edition

Uses Rust edition 2024 with Tokio async runtime.
