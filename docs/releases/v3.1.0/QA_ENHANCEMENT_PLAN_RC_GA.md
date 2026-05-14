# SQLRustGo v3.1.0 QA Enhancement Plan (RC to GA)

> **Version**: 1.0
> **Date**: 2026-05-14
> **Phase**: RC → GA
> **CMMI Target**: Level 3
> **Status**: Industrial QA Skeleton Establishment
>
> **Issue Tracking**:
> - [Issue #859 - QA 改进总控](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/859)
> - [Issue #871 - 性能优化总控](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/871)
> - [Issue #873 - 测试覆盖增强总控](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/873)
> - [Issue #880 - RC 阶段完成总控](http://192.168.0.252:3000/openclaw/sqlrustgo/issues/880)

---

## 1. Overview

### 1.1 Purpose

This document defines the QA enhancement plan for SQLRustGo v3.1.0, transitioning from Release Candidate (RC) to General Availability (GA). The plan establishes an industrial-grade QA skeleton inspired by PostgreSQL, CockroachDB, and TiDB best practices.

### 1.2 Current State Assessment

| Dimension | Current Level | Target Level | Gap |
|-----------|---------------|--------------|-----|
| Functional Completeness | RC | GA | Full SQL standard compliance |
| QA Engineering | Beta | Industrial QA Skeleton | Process standardization |
| CI/CD | Alpha-Beta | Beta | Automated quality gates |
| Code Coverage | ~70% | ≥85% | +15% improvement |
| Static Analysis | Basic Clippy | Full toolchain | Add Miri, sanitizers |
| Security Scanning | Manual | Automated | Integrate cargo-audit |
| Benchmark | Basic | TPC-C/H baseline | Automate performance tests |
| Process Engineering | Alpha | Beta | CMMI Level 3 practices |

### 1.3 Release Objectives

```
v3.1.0 RC → GA Quality Targets
┌────────────────────────────────────────────────────────────┐
│  Gate      │  Coverage  │  Performance      │  Stability │
│  R-Gate    │  ≥85%      │  TPC-H SF=1 22/22 │  No P0/P1  │
│  G-Gate    │  ≥85%      │  Point Select     │  No P0/P1  │
│            │            │  ≥10K QPS         │            │
└────────────────────────────────────────────────────────────┘
```

---

## 2. Test Standardization

### 2.1 sqllogictest Integration

**Status**: Partial implementation
**Target**: Complete PostgreSQL-compatible test suite

#### 2.1.1 Test Directory Structure

```
tests/
├── sql/
│   ├── syntax/
│   │   ├── select.test
│   │   ├── where.test
│   │   ├── join.test
│   │   ├── subquery.test
│   │   └── order_by.test
│   ├── dml/
│   │   ├── insert.test
│   │   ├── update.test
│   │   └── delete.test
│   ├── ddl/
│   │   ├── create_table.test
│   │   ├── index.test
│   │   └── alter.test
│   ├── transaction/
│   │   ├── basic_txn.test
│   │   ├── mvcc_snapshot.test
│   │   └── wal_recovery.test
│   ├── types/
│   │   ├── numeric.test
│   │   ├── string.test
│   │   └── datetime.test
│   └── compatibility/
│       └── pg_*.test
├── integration/
│   ├── mysql_protocol.test
│   └── concurrent_access.test
└── regression/
    └── known_bugs.test
```

#### 2.1.2 Test File Example (MVCC Snapshot Isolation)

```sql
-- tests/sql/transaction/mvcc_snapshot.test
-- name: mvcc_snapshot.test
-- description: MVCC snapshot isolation test
-- group: [transaction]

statement ok
CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER);

statement ok
INSERT INTO accounts VALUES (1, 100), (2, 200);

# Session 1: Start transaction with snapshot
query I
BEGIN;
----
ok

query I
SELECT * FROM accounts WHERE id = 1;
----
1
100

# Session 2: Concurrent update (should not be visible to Session 1)
statement ok
BEGIN;

statement ok
UPDATE accounts SET balance = 150 WHERE id = 1;

query I
SELECT * FROM accounts WHERE id = 1;
----
1
150

statement ok
COMMIT;

# Session 1: Should still see old value (snapshot isolation)
query I
SELECT * FROM accounts WHERE id = 1;
----
1
100

statement ok
COMMIT;
```

#### 2.1.3 Implementation Requirements

| Component | Status | Action Required |
|-----------|--------|-----------------|
| sqllogictest crate | Implemented | Verify compatibility with PostgreSQL format |
| Test runner | Implemented | Add parallel execution support |
| Test files | 50+ | Create 200+ additional tests |
| Coverage integration | Partial | Integrate with llvm-cov |

### 2.2 Test Execution Framework

#### 2.2.1 Cargo Test Integration

```toml
# Cargo.toml
[dev-dependencies]
sqllogictest = "0.9"

[[test]]
name = "sqllogictest"
path = "tests/sqllogictest_runner.rs"
```

#### 2.2.2 Test Runner Implementation

```rust
// tests/sqllogictest_runner.rs
use sqllogictest::{ColumnType, DBOutput, Runner};

pub struct SQLRustGoDB {
    // Implementation
}

impl sqllogictest::DB for SQLRustGoDB {
    fn run(&self, sql: &str) -> DBOutput {
        match self.query(sql) {
            Ok(result) => result.into(),
            Err(e) => DBOutput::Error(format!("{}", e)),
        }
    }

    fn engine_name(&self) -> &str {
        "sqlrustgo"
    }
}

#[test]
fn test_all_sql_files() {
    let mut runner = Runner::new(SQLRustGoDB::new());
    runner.run_file("tests/sql/syntax/select.test").unwrap();
    // ... additional files
}
```

---

## 3. Static Analysis Tools

### 3.1 Clippy (P0 - Required)

**Status**: Basic configuration
**Target**: Strict industrial-grade linting

#### 3.1.1 Clippy Configuration

```toml
# clippy.toml (create in project root)
msrv = "1.80"
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 6
type-complexity-threshold = 500
single-char-binding-names-threshold = 3
disallowed-names = ["foo", "bar", "baz", "quux"]

# Enable specific lints
enable.await-holding-reentrant-lock
```

#### 3.1.2 Cargo.toml Lint Configuration

```toml
# Cargo.toml
[lints.rust]
unsafe_code = "deny"
unused = "deny"
rust_2018_idioms = "deny"
```

#### 3.1.3 CI Integration

```yaml
# .github/workflows/clippy.yml
name: Clippy Lint

on: [pull_request, push]

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-features --all-targets -- -D warnings
```

### 3.2 Miri (P1 - UB Detection)

**Status**: Not integrated
**Target**: Nightly UB detection for memory-critical code

#### 3.2.1 Installation

```bash
rustup +nightly component add miri
```

#### 3.2.2 Target Modules for Miri Testing

| Module | Priority | Reason |
|--------|----------|--------|
| MVCC snapshot management | P0 | Memory safety critical |
| WAL read/write | P0 | Data integrity |
| Memory allocator | P1 | Low-level operations |
| Transaction manager | P1 | Concurrent access |

#### 3.2.3 CI Integration

```yaml
# .github/workflows/miri.yml
name: Miri UB Detection

on:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday
  push:
    branches: [main, develop/v3.1.0]

jobs:
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: miri
      - name: Run Miri
        env:
          MIRIFLAGS: "-Zmiri-disable-isolation"
        run: cargo +nightly miri test --lib
```

### 3.3 Sanitizers (P1 - Runtime Detection)

**Status**: Not integrated
**Target**: Automated sanitizer testing in CI

#### 3.3.1 Sanitizer Types

| Sanitizer | Purpose | Flags | Priority |
|-----------|---------|-------|----------|
| AddressSanitizer (ASan) | Memory errors | `-Z sanitizer=address` | P0 |
| UndefinedBehaviorSanitizer (UBSan) | UB detection | `-Z sanitizer=undefined` | P0 |
| ThreadSanitizer (TSan) | Data races | `-Z sanitizer=thread` | P1 |
| LeakSanitizer (LSan) | Memory leaks | `-Z sanitizer=leak` | P1 |

#### 3.3.2 CI Integration

```yaml
# .github/workflows/sanitizers.yml
name: Sanitizers

on:
  schedule:
    - cron: '0 3 * * *'  # Nightly
  push:
    branches: [main]

jobs:
  asan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run ASan
        env:
          RUSTFLAGS: "-Z sanitizer=address"
        run: cargo +nightly test -Z sanitizer=address

  ubsan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Run UBSan
        env:
          RUSTFLAGS: "-Z sanitizer=undefined"
        run: cargo +nightly test -Z sanitizer=undefined
```

---

## 4. Security Scanning

### 4.1 cargo-audit Integration

**Status**: Manual execution
**Target**: Automated security scanning in CI

#### 4.1.1 Installation and Usage

```bash
cargo install cargo-audit
cargo audit
```

#### 4.1.2 CI Integration

```yaml
# .github/workflows/security.yml
name: Security Scanning

on: [push, pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run cargo-audit
        run: cargo audit
```

#### 4.1.3 Configuration

```toml
# .cargo/audit.toml
[advisory]
# Ignore specific advisories with justification
ignore = [
    # "RUSTSEC-XXXX-XXXX",  # Reason for ignoring
]
```

### 4.2 Dependency Vulnerability Scanning

| Check | Tool | Frequency | Target |
|-------|------|-----------|--------|
| Crate vulnerabilities | cargo-audit | Every PR | Deny high/critical |
| Outdated dependencies | cargo-outdated | Weekly | Warning |
| License compliance | cargo-deny | Every PR | Deny copyleft |

#### 4.2.1 cargo-deny Configuration

```toml
# deny.toml
[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
copyleft = "deny"

[bans]
deny = [" GPL-2.0", "LGPL-2.1"]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

---

## 5. Benchmark Integration

### 5.1 TPC-H Benchmark

**Status**: Manual execution
**Target**: Automated TPC-H SF=1 testing at R-Gate

#### 5.1.1 TPC-H Test Requirements

| Query | SF=0.1 | SF=1 | Status |
|-------|--------|------|--------|
| Q1 | ✓ | ✓ | Must pass |
| Q2 | ✓ | ✓ | Must pass |
| ... | ... | ... | ... |
| Q22 | ✓ | ✓ | Must pass |

#### 5.1.2 CI Integration

```yaml
# .github/workflows/tpch.yml
name: TPC-H Benchmark

on:
  schedule:
    - cron: '0 4 * * *'  # Nightly
  push:
    branches: [main, develop/v3.1.0]

jobs:
  tpch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Prepare TPC-H data
        run: |
          # Generate SF=1 data
          ./scripts/benchmark/tpch-gen.sh sf=1
      - name: Run TPC-H
        run: |
          ./scripts/benchmark/tpch-run.sh sf=1
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: tpch-results
          path: results/
```

### 5.2 Point Select Benchmark

**Status**: Basic benchmark
**Target**: Point Select ≥10,000 QPS at G-Gate

#### 5.2.1 Performance Targets (G-Gate)

| Metric | Target | Current | Gap |
|--------|--------|---------|-----|
| Point Select QPS | ≥10,000 | ~8,000 | +2,000 |
| UPDATE QPS | ≥5,000 | ~4,000 | +1,000 |
| DELETE QPS | ≥2,000 | ~1,500 | +500 |
| TPC-H SF=1 | 22/22 | 20/22 | +2 |

#### 5.2.2 Benchmark Script

```bash
#!/bin/bash
# scripts/benchmark/point_select.sh

TABLE_SIZE=100000
THREADS=4
DURATION=60

echo "Point Select Benchmark"
echo "====================="
echo "Table size: $TABLE_SIZE"
echo "Threads: $THREADS"
echo "Duration: ${DURATION}s"

cargo bench --package sqlrustgo -- point_select --threads $THREADS --time $DURATION
```

---

## 6. CI/CD Quality Gates

### 6.1 R-Gate Quality Requirements

| Check | Requirement | Command | Timeout |
|-------|-------------|---------|---------|
| Compilation | No errors | `cargo build --release --workspace` | 10min |
| Unit Tests | 100% pass | `cargo test --all-features` | 30min |
| Clippy | Zero warnings | `cargo clippy --all-features -- -D warnings` | 10min |
| Format | No diffs | `cargo fmt --all -- --check` | 5min |
| Coverage | ≥85% | `cargo llvm-cov --all-features --lcov` | 20min |
| cargo-audit | No vulnerabilities | `cargo audit` | 5min |
| TPC-H SF=1 | 22/22 pass | `./scripts/tpch.sh sf=1` | 60min |
| Point Select | ≥10,000 QPS | `cargo bench -- point_select` | 10min |

### 6.2 G-Gate Quality Requirements

All R-Gate requirements plus:

| Check | Requirement | Command | Timeout |
|-------|-------------|---------|---------|
| UPDATE QPS | ≥5,000 | `cargo bench -- update_simple` | 10min |
| DELETE QPS | ≥2,000 | `cargo bench -- delete_simple` | 10min |
| SQL Corpus | ≥98% | `cargo test -p sqlrustgo-sql-corpus` | 30min |
| Stability | B-S1~B-S6 PASS | `cargo test --test stability` | 60min |
| MySQL Protocol | Connection OK | Docker test | 10min |

### 6.3 GitHub Actions Workflow

```yaml
# .github/workflows/quality-gate.yml
name: Quality Gate

on:
  pull_request:
    branches: [main, develop/v3.1.0]

jobs:
  r-gate:
    name: R-Gate Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --release --workspace
      - name: Test
        run: cargo test --all-features
      - name: Clippy
        run: cargo clippy --all-features -- -D warnings
      - name: Format check
        run: cargo fmt --all -- --check
      - name: Coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Security audit
        run: cargo audit
      - name: TPC-H SF=1
        run: ./scripts/benchmark/tpch.sh sf=1

  performance:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Point Select
        run: cargo bench -- point_select
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: bench-results
          path: target/criterion/
```

---

## 7. Test Coverage Plan

### 7.1 Coverage Targets by Module

| Module | Current | v3.1.0 Target | Priority |
|--------|---------|--------------|----------|
| parser | 85% | 90% | P0 |
| executor | 65% | 80% | P0 |
| storage | 70% | 85% | P0 |
| transaction | 75% | 85% | P0 |
| network | 60% | 75% | P1 |
| optimizer | 40% | 60% | P1 |

### 7.2 Coverage Improvement Actions

1. **Parser Module (P0)**
   - Add negative test cases for invalid SQL
   - Add boundary condition tests for all data types
   - Target: 90% line coverage

2. **Executor Module (P0)**
   - Add JOIN operation tests
   - Add aggregate function tests
   - Add NULL handling tests
   - Target: 80% line coverage

3. **Storage Module (P0)**
   - Add page-level operation tests
   - Add B-tree operation tests
   - Add WAL recovery tests
   - Target: 85% line coverage

4. **Transaction Module (P0)**
   - Add MVCC isolation level tests
   - Add deadlock detection tests
   - Add two-phase commit tests
   - Target: 85% line coverage

---

## 8. Process Engineering (CMMI Level 3)

### 8.1 QA Process Definition

#### 8.1.1 Requirements

```
┌─────────────────────────────────────────────────────────────┐
│                    REQM - Requirements Management             │
├─────────────────────────────────────────────────────────────┤
│ • Elicitation: User stories, SQL standard compliance        │
│ • Analysis: Feasibility, risk assessment                     │
│ • Specification: Testable requirements in GitHub Issues      │
│ • Verification: Peer review of requirements                  │
│ • Change management: Impact analysis for requirement changes │
└─────────────────────────────────────────────────────────────┘
```

#### 8.1.2 Test Management

```
┌─────────────────────────────────────────────────────────────┐
│                    TEST - Test Management                    │
├─────────────────────────────────────────────────────────────┤
│ • Test planning: v3.1.0 test plan documented                │
│ • Test monitoring: Weekly coverage reports                    │
│ • Test completion: Exit criteria defined per gate            │
│ • Test object evaluation: Results documented in artifacts    │
└─────────────────────────────────────────────────────────────┘
```

### 8.2 Artifact Repository

| Artifact | Location | Review Frequency |
|----------|----------|-----------------|
| Test Plan | `docs/releases/v3.1.0/TEST_PLAN.md` | Per release |
| Coverage Report | `docs/releases/v3.1.0/COVERAGE_ANALYSIS.md` | Weekly |
| QA Metrics | `docs/releases/v3.1.0/QA_METRICS.md` | Per sprint |
| Test Cases | `tests/sql/` | Continuous |
| Benchmark Results | `benchmarks/` | Per CI run |

### 8.3 Quality Metrics

| Metric | Baseline | v3.1.0 Target | Measurement |
|--------|----------|--------------|-------------|
| Code coverage | 70% | 85% | llvm-cov |
| Test pass rate | 95% | 100% | cargo test |
| Bug escape rate | 15% | <5% | Production bugs |
| Security vulnerabilities | 3 | 0 | cargo audit |
| Performance regression | 10% | <5% | TPC-H benchmark |

---

## 9. Implementation Timeline

### 9.1 v3.1.0 RC→GA Timeline

```
Week 1-2: Test Standardization
├── Add 100+ sqllogictest cases
├── Configure Clippy strict mode
├── Integrate cargo-audit in CI
└── Verify TPC-H SF=1 all queries

Week 3-4: Static Analysis & Security
├── Enable Miri testing for MVCC/WAL
├── Configure sanitizers (ASan, UBSan)
├── Add cargo-deny license checking
└── Complete security scan automation

Week 5-6: Performance & Benchmark
├── Optimize Point Select to ≥10K QPS
├── Verify TPC-H SF=1 stability
├── Automate benchmark reporting
└── Final G-Gate verification
```

### 9.2 Deliverables

| Deliverable | Due | Owner |
|------------|-----|-------|
| sqllogictest 200+ cases | Week 2 | QA Team |
| Clippy strict config | Week 1 | Dev Team |
| Miri integration | Week 3 | Dev Team |
| Sanitizer CI | Week 4 | CI Team |
| cargo-audit automation | Week 2 | CI Team |
| TPC-H SF=1 automation | Week 4 | QA Team |
| Performance optimization | Week 6 | Performance Team |
| G-Gate verification | Week 6 | QA Team |

---

## 10. Success Criteria

### 10.1 R-Gate Entry Criteria

- [ ] All R-Gate checks implemented in CI
- [ ] TPC-H SF=1: 22/22 queries pass
- [ ] Code coverage ≥85%
- [ ] Zero high/critical security vulnerabilities
- [ ] Zero P0/P1 bugs open

### 10.2 G-Gate Entry Criteria

- [ ] All G-Gate checks implemented in CI
- [ ] Point Select QPS ≥10,000
- [ ] UPDATE QPS ≥5,000
- [ ] DELETE QPS ≥2,000
- [ ] SQL Corpus ≥98%
- [ ] All stability tests pass
- [ ] MySQL protocol test passes
- [ ] Human Architect sign-off obtained

### 10.3 Definition of Done

```
v3.1.0 GA = R-Gate PASS + G-Gate PASS + Architect Approval + Documentation Complete
```

---

## 11. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Performance target not met | Medium | High | Start optimization early (Week 3) |
| Coverage gap in executor | Medium | Medium | Prioritize executor tests (Week 1-2) |
| Miri test timeout | Low | Medium | Limit to critical modules |
| TPC-H query failures | Low | High | Use PostgreSQL as reference |

---

## 12. Appendix

### 12.1 Reference Documents

- PostgreSQL Testing: https://www.postgresql.org/docs/current/regress-ion.html
- DuckDB sqllogictest: https://duckdb.org/docs/testing
- SQLite TH3: https://sqlite.org/testing.html
- Cargo Clippy: https://doc.rust-lang.org/clippy/
- Miri: https://github.com/rust-lang/miri
- TPC-H Specification: https://www.tpc.org/tpch/

### 12.2 Related Documents

- `TEST_IMPROVEMENT_ROADMAP.md` - Overall testing strategy
- `TOOL_INTEGRATION_GUIDE.md` - Tool integration details
- `VERSION_LIFECYCLE_MANAGEMENT.md` - Release process
- `GATE_SPEC_MASTER.md` - Quality gate specifications
