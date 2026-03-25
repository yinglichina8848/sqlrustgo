# SQLRustGo Benchmark Report

## Overview

This document reports the performance benchmarks for SQLRustGo database operations.

## Test Environment

- **Platform**: macOS (Darwin)
- **Rust Version**: 1.75+ (Edition 2021)
- **Benchmark Framework**: Criterion.rs 0.5
- **Storage**: MemoryStorage (in-memory)

## Benchmark Results

### 1. INSERT Performance

| Operation | Dataset Size | Time | Throughput |
|-----------|--------------|------|------------|
| INSERT | 1,000 | 27.96 µs/row | ~35,778 rows/s |
| INSERT | 10,000 | 29.93 µs/row | ~33,416 rows/s |
| INSERT | 100,000 | 30.65 µs/row | ~32,629 rows/s |

### 2. Batch INSERT Performance

| Batch Size | Total Rows | Time (total) | Throughput |
|------------|------------|--------------|------------|
| 10 | 10,000 | 389.40 µs/batch | ~25,678 rows/s |
| 100 | 10,000 | 317.46 µs/batch | ~31,499 rows/s |
| 1,000 | 10,000 | 304.63 µs/batch | ~32,827 rows/s |

### 3. Aggregate Performance

| Operation | Dataset Size | Time |
|-----------|--------------|------|
| COUNT(*) | 100 | ~870 ns |
| COUNT(*) | 1,000 | ~1.08 µs |
| COUNT(*) | 10,000 | ~1.08 µs |
| SUM(amount) | 100 | ~1.07 µs |
| SUM(amount) | 1,000 | ~1.08 µs |
| SUM(amount) | 10,000 | ~1.10 µs |
| AVG(amount) | 100 | ~2.16 µs |
| AVG(amount) | 1,000 | ~2.16 µs |

### 4. Lexer Performance

| Operation | Time |
|-----------|------|
| Simple SELECT | 1.26 µs |
| Medium SELECT | 2.07 µs |
| Complex JOIN | 5.88 µs |
| INSERT | 3.53 µs |
| UPDATE | 2.18 µs |
| DELETE | 1.68 µs |
| CREATE TABLE | 2.76 µs |
| DROP TABLE | 379 ns |

### 5. Parser Performance

| Operation | Time |
|-----------|------|
| Simple SELECT | 1.62 µs |
| SELECT with WHERE | 2.47 µs |
| JOIN | 3.67 µs |
| ORDER BY | 1.52 µs |
| INSERT | 1.04 µs |
| UPDATE | 1.85 µs |
| DELETE | 1.22 µs |
| CREATE TABLE | 1.60 µs |
| Batch 100 SELECT | 242 µs |

### 6. End-to-End Performance

| Operation | Time |
|-----------|------|
| Parse + Execute SELECT | 1.50 µs |
| Execute INSERT | 1.36 µs |
| Execute COUNT | 1.82 µs |

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --bench bench_insert
cargo bench --bench bench_scan
cargo bench --bench bench_aggregate
```

## Key Findings

1. **Linear Scaling**: INSERT operations show O(n) complexity
2. **Memory Storage**: Current implementation uses in-memory storage for maximum performance
3. **Batch Operations**: Batch INSERT shows better throughput than single-row INSERT
4. **Parser Efficiency**: Simple queries parse in ~1-2 µs
5. **Aggregate Speed**: COUNT/SUM/AVG operations complete in ~1-2 µs

## Test Coverage

| Module | Coverage |
|--------|----------|
| Total | **80.11%** |
| planner/planner.rs | 97.53% |
| planner/optimizer.rs | 90.96% |
| optimizer/rules.rs | 81.96% |
| executor/local_executor.rs | 69.47% |

## Future Improvements

- [ ] Add disk-based storage benchmarks
- [ ] Add index performance benchmarks
- [ ] Add JOIN operation benchmarks
- [ ] Add concurrent access benchmarks
