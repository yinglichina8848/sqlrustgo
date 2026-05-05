# Sysbench OLTP Benchmark Report - SQLRustGo v2.9.0

## Executive Summary

This report documents the Sysbench OLTP benchmark infrastructure established for SQLRustGo v2.9.0 and presents initial performance results.

**Target**: ≥ 10K QPS
**Current**: ~2,000 QPS (point select, 8-32 threads)
**Gap**: 5x performance improvement needed

## 1. Test Environment

| Component | Version/Details |
|-----------|-----------------|
| SQLRustGo | v2.9.0 (develop/v2.9.0) |
| MySQL Server | v2.6.0+ (mysql-server crate) |
| sysbench | 1.0.20 |
| OS | Ubuntu 24.04 (Linux) |
| Rust | 1.85+ |
| Test Database | sbtest |
| Table Size | 10,000 rows |
| Connection | MySQL Wire Protocol (port 3306) |

### Reference Hardware (for comparison)

| Component | MySQL/PG/SQLite Tests |
|-----------|----------------------|
| CPU | Intel Xeon Gold 6138 @ 2.00GHz (80 核) |
| Memory | 409 GB DDR4 |
| Disk | NVMe SSD 1.9TB |

## 2. Benchmark Results

### 2.1 Point Select (Primary Key Lookup)

| Threads | QPS | TPS | Avg Latency (ms) | Max Latency (ms) |
|---------|-----|-----|------------------|------------------|
| 1 | ~1,850 | ~1,850 | 0.54 | 8.32 |
| 8 | ~1,840 | ~1,840 | 4.34 | 43.76 |
| 32 | ~1,975 | ~1,975 | 16.19 | 305.61 |

**Observation**: QPS remains relatively constant (~2,000) regardless of thread count, suggesting the bottleneck is in query execution rather than concurrency.

### 2.2 Comparison with Other Databases (32 threads)

| Database | Point Select TPS | Relative to MySQL |
|----------|------------------|-------------------|
| PostgreSQL 16 | 285,128 | 127% |
| MySQL 8.0 | 224,931 | 100% |
| SQLite 3.45 | 13,617 | 6% |
| **SQLRustGo** | **~2,000** | **<1%** |

### 2.3 Known Limitations

1. **No Transaction Support**: OLTP read_write and other transaction-heavy tests fail with "No transaction in progress"
2. **Limited Concurrency Scaling**: QPS does not increase with additional threads beyond 8
3. **High Latency at Scale**: Max latency increases significantly with 32 threads (305ms vs 43ms at 8 threads)

## 3. Infrastructure

### 3.1 Scripts

| Script | Purpose |
|--------|---------|
| `scripts/sysbench/point_select.sh` | Run point select benchmark |

### 3.2 Usage

```bash
# Run point select test with default settings (8 threads, 30s)
./scripts/sysbench/point_select.sh

# Run with custom thread count and duration
./scripts/sysbench/point_select.sh 32 60

# Run with custom connection parameters
HOST=192.168.1.100 PORT=3306 USER=mysql PASSWORD=mysql ./scripts/sysbench/point_select.sh
```

### 3.3 MySQL Server Authentication

The mysql-server uses `mysql_native_password` authentication. Default users:

| User | Password | Notes |
|------|----------|-------|
| root | (empty) | Default root user |
| mysql | mysql | Recommended for testing |

## 4. Path to 10K QPS

### 4.1 Identified Bottlenecks

1. **Query Parser**: Each query is parsed from scratch
2. **No Query Caching**: Prepared statements not cached
3. **Single-threaded Executor**: Query execution is sequential
4. **Memory Storage**: No optimization for in-memory workloads

### 4.2 Required Optimizations

| Optimization | Estimated Impact | Priority |
|--------------|------------------|----------|
| Query plan caching | 2-3x QPS | P0 |
| Prepared statement support | 2-4x QPS | P0 |
| SIMD acceleration in executor | 2-5x QPS | P1 |
| Connection pooling | 1.5-2x QPS | P1 |
| Batch query processing | 2-3x QPS | P2 |

## 5. Acceptance Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| Sysbench OLTP baseline established | ✅ Complete | Infrastructure ready |
| ≥ 10K QPS on point select | ❌ Not met | Currently ~2K QPS |
| Scripts for reproducible testing | ✅ Complete | point_select.sh created |
| Documentation | ✅ Complete | This report |

## 6. Next Steps

1. **E-01a**: Implement query plan caching to reduce parse overhead
2. **E-01b**: Add prepared statement support with plan reuse
3. **E-01c**: Profile and optimize hot path in executor
4. **E-01d**: Re-benchmark after optimizations

---

*Report generated: 2026-05-02*
*Branch: feature/e01-sysbench*