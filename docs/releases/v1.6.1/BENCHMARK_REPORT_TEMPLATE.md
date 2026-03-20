# SQLRustGo v1.6.1 Performance Evaluation Report

> **版本**: v1.6.1
> **发布日期**: TBD
> **状态**: Draft

---

## 1. Abstract

This report evaluates the performance of SQLRustGo v1.6.1 against PostgreSQL and SQLite.

We focus on:
- OLTP throughput (YCSB-like workload)
- OLAP query performance (TPC-H subset)
- Latency distribution (P50/P95/P99)

All benchmarks are conducted under reproducible and controlled conditions.

---

## 2. System Overview

### 2.1 SQLRustGo

| Component | Description |
|-----------|-------------|
| Architecture | Volcano + MVCC + WAL |
| Isolation Level | READ COMMITTED |
| Storage | B+Tree + Buffer Pool |
| Version | v1.6.1 |

### 2.2 PostgreSQL (Baseline)

| Component | Description |
|-----------|-------------|
| Version | 15+ |
| Storage | heap + MVCC |
| WAL | enabled |

### 2.3 SQLite (Reference)

| Component | Description |
|-----------|-------------|
| Mode | WAL |
| Type | single-node embedded |

---

## 3. Benchmark Setup

### 3.1 Hardware Environment

| Item | Value |
|------|-------|
| CPU | Apple M2 / x86_64 |
| Cores | 8 |
| RAM | 16GB |
| Disk | SSD |

### 3.2 Software Environment

| Item | Value |
|------|-------|
| OS | macOS / Linux |
| Rust | 1.75+ |
| PostgreSQL | 15 |
| SQLite | 3.x |

### 3.3 Configuration

#### PostgreSQL
```sql
shared_buffers = 1GB
synchronous_commit = off
```

#### SQLite
```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
```

#### SQLRustGo
```yaml
query_cache: OFF
wal: ON
buffer_pool: ENABLED
```

---

## 4. Workloads

### 4.1 OLTP (YCSB-like)

| Operation | Ratio |
|-----------|-------|
| Read | 50% |
| Update | 30% |
| Insert | 10% |
| Scan | 10% |

### 4.2 OLAP (TPC-H Subset)

- Q1: Aggregation with grouping
- Q3: JOIN with filter
- Q6: Range filter
- Q10: Complex JOIN

---

## 5. Results

### 5.1 OLTP Throughput

| System | TPS | Notes |
|--------|-----|-------|
| SQLRustGo (Embedded) | TBD | |
| SQLRustGo (TCP) | TBD | |
| PostgreSQL | TBD | |
| SQLite | TBD | |

### 5.2 Latency Distribution

| System | P50 (µs) | P95 (µs) | P99 (µs) |
|--------|----------|----------|----------|
| SQLRustGo TCP | TBD | TBD | TBD |
| PostgreSQL | TBD | TBD | TBD |

### 5.3 OLAP Queries

| Query | SQLRustGo (ms) | PostgreSQL (ms) |
|-------|----------------|-----------------|
| Q1 | TBD | TBD |
| Q3 | TBD | TBD |
| Q6 | TBD | TBD |
| Q10 | TBD | TBD |

---

## 6. Analysis

### 6.1 Embedded vs TCP

Analysis of network and protocol overhead.

### 6.2 SQLRustGo vs PostgreSQL

Analysis:
- CPU bound?
- Lock contention?
- WAL bottleneck?

### 6.3 Latency Tail (P99)

Higher P99 latency indicates contention in:
- Lock manager
- WAL flush
- Executor

### 6.4 Performance Breakdown

```json
{
  "lock_wait_ratio": 0.XX,
  "wal_flush_ratio": 0.XX,
  "executor_cpu_ratio": 0.XX
}
```

---

## 7. Limitations

- Only READ COMMITTED isolation level
- Limited SQL support (no UNION)
- No SIMD optimization
- Small dataset (SF=1)

---

## 8. Conclusion

SQLRustGo achieves [competitive / moderate / below-expectation] performance in OLTP workloads.

**Future work**:
- Serializable isolation
- SIMD execution
- Distributed architecture

---

## Appendix: Raw Data

[To be filled with actual benchmark results]

---

*Report Template - v1.6.1*
