# AGENTS.md - Agent Guidelines for SQLRustGo

SQLRustGo is a Rust SQL-92 compliant database with lexer, parser, executor, B+ Tree storage, WAL, and network protocol.

---

## Build/Lint/Test Commands

```bash
# Format check
cargo fmt --all -- --check

# Lint (warnings as errors)
cargo clippy --all-targets -- -D warnings

# Build
cargo build --all

# Run all tests
cargo test --all

# Run single test
cargo test test_name

# Run tests in module
cargo test module_name

# Run benchmarks
cargo bench

# Coverage
cargo llvm-cov --all-features
```

---

## Code Style

### Formatting
- Use `cargo fmt` (4 spaces, max 100 chars, trailing commas)

### Imports (grouped by category)
```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::parser::Statement;
```

### Naming
- Types: `PascalCase` (struct, enum)
- Functions/vars: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling
- Use `SqlResult<T> = Result<T, SqlError>`
- Use `thiserror` for error enums
- Implement `From` for conversions
- Use `ok_or_else` or `map_err` for context

**DON'T:** `unwrap()`, `panic!`, `expect()` (except tests)

```rust
// Good
let table = self.storage.get_table(&name)
    .ok_or_else(|| SqlError::TableNotFound(name.clone()))?;

// Bad
let table = self.storage.get_table(&name).unwrap();
```

### Types
- `i64` = INTEGER, `f64` = FLOAT
- `String` = TEXT (not `&str` in public APIs)
- `Vec<u8>` = BLOB, `bool` = BOOLEAN
- Explicit: `Vec<T>`, `HashMap<K,V>`, `HashSet<T>`

### Documentation
- `///` for public items
- `//!` for module tops
- Include examples in doc comments

```rust
/// Execute a SQL statement
pub fn execute(&mut self, statement: Statement) -> SqlResult<ExecutionResult>

//! Storage module
```

### Async
- Use `tokio` for async
- `#[tokio::test]` for async tests

### Testing
- All new logic needs tests
- Descriptive names: `test_feature_scenario`
- Use temp dirs for isolation
- Clean up in tests

---

## PR/Commit Conventions

```
type(scope): summary
```

Types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `chore`
Scopes: `parser`, `executor`, `planner`, `optimizer`, `storage`, `network`, `auth`, `ci`

Examples:
- `feat(auth): implement basic authentication`
- `fix(executor): remove unwrap panic`

---

## Coverage Requirements
- No coverage decrease
- Core modules (parser/executor/planner) >= 80%

---

## Dependencies
- `tokio` - async runtime
- `thiserror` - error handling
- `anyhow` - general errors
- `serde`/`serde_json` - serialization
- `log`/`env_logger` - logging
- `bytes` - network buffers

---

## Project Structure

```
src/
  auth/       - Authentication
  executor/   - Query execution
  lexer/      - Tokenizer
  network/    - Network protocol
  parser/     - SQL parser
  storage/    - B+ Tree, buffer pool, WAL
  transaction/- Transaction management
  types/      - Value, SqlError
```
