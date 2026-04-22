# Cargo-Nextest Evaluation Report

Date: 2026-04-22
Status: Recommended for v2.7.0 Phase 4

## Overview

cargo-nextest is a fast, parallel test runner for Rust that could replace or supplement tarpaulin for faster test execution.

## Comparison

| Feature | cargo-tarpaulin | cargo-nextest |
|---------|-----------------|---------------|
| Coverage | ✅ Yes | ❌ No (yet) |
| Speed | Slow (single-threaded) | ✅ Fast (parallel) |
| Test Filtering | Basic | ✅ Advanced |
| Output | Coverage reports | ✅ Better UI |
| CI Integration | ✅ Yes | ✅ Yes |

## Nextest + Tarpaulin Strategy

**Recommended approach for v2.7.0:**

1. **PR Feedback Loop**: Use `cargo nextest run` for fast test execution during development
2. **Coverage**: Keep `cargo tarpaulin` for coverage reports (run less frequently)
3. **Parallel Coverage**: Use GitHub Actions matrix to parallelize tarpaulin runs

## Installation

```bash
cargo install cargo-nextest
```

## Usage

```bash
cargo nextest run
cargo nextest run --test-threads 8
cargo nextest run -E 'package(sqlrustgo-executor)'
```

## Recommendation

**Defer full nextest integration to v2.7.0 Phase 4** as it's not critical path.

Current implementation:
- ✅ Parallel coverage workflow created
- ✅ Performance tests separated
- ✅ Incremental coverage support added

These changes address the immediate timeout issues without requiring nextest adoption.
