# v3.1.0 Performance Targets

> **Version**: 3.1.0  
> **Date**: 2026-05-11

---

## Performance Goals

| Metric | v3.0.0 GA | v3.1.0 Target | Change |
|--------|-----------|----------------|--------|
| Point SELECT QPS | 398K | **≥400K** | +0.5% |
| UPDATE QPS | 532K | **≥550K** | +3.4% |
| DELETE QPS | 706K | **≥700K** | -0.9% |
| INSERT QPS | ~400K | **≥450K** | +12.5% |
| Complex WHERE QPS | ~1220 | **≥5000** | +310% |

---

## QPS Benchmarks (8/8 within 5% of baseline)

| Benchmark | v3.0.0 GA | v3.1.0 Target | Regression Limit |
|-----------|-----------|----------------|-----------------|
| `simple_select` | 198K (debug) | ≥400K (release) | ≤5% |
| `insert` | 14.7K | ≥50K | ≤5% |
| `update` | 30.4K | ≥550K | ≤5% |
| `delete` | 51.5K | ≥700K | ≤5% |
| `join` | 48.4K | ≥50K | ≤5% |
| `aggregation` | 671K | ≥600K | ≤5% |
| `order_by` | 47.3K | ≥50K | ≤5% |
| `complex_where` | 1.2K | ≥5K | ≤5% |

> **Note**: v3.0.0 QPS measured in debug mode (2-3x lower than release). v3.1.0 all benchmarks use `--release`.

---

## TPC-H Benchmarks

| Scale Factor | v3.0.0 | v3.1.0 Target | Status |
|-------------|---------|----------------|--------|
| SF=0.1 (100MB) | 22/22, ~10.9s | 22/22, <10s | ✅ |
| SF=1 (1GB) | OOM | 22/22, <60s | 🟡 In Progress |

### TPC-H SF=1 Query Time Targets

| Query | v3.0.0 | v3.1.0 Target |
|-------|---------|----------------|
| Q1 (Pricing) | OOM | <5s |
| Q2 (Minimum) | OOM | <3s |
| Q3 (Shipping) | OOM | <5s |
| Q4 (Order Priority) | OOM | <3s |
| Q5 (Local Supplier) | OOM | <5s |
| Q6 (Forecast) | OOM | <3s |
| Q7 (Volume) | OOM | <5s |
| Q8 (National) | OOM | <5s |
| Q9 (Product) | OOM | <8s |
| Q10 (Returned) | OOM | <5s |
| Q11 (Stock) | OOM | <3s |
| Q12 (Shipping) | OOM | <5s |
| Q13 (Customer) | OOM | <5s |
| Q14 (Promotion) | OOM | <3s |
| Q15 (Top) | OOM | <5s |
| Q16 (Parts/Supplier) | OOM | <3s |
| Q17 (Lineitem) | OOM | <5s |
| Q18 (Large) | OOM | <10s |
| Q19 (Discount) | OOM | <5s |
| Q20 (Order) | OOM | <5s |
| Q21 (Supplier) | OOM | <8s |
| Q22 (Geographic) | OOM | <5s |

---

## Memory Targets

| Metric | v3.0.0 | v3.1.0 Target |
|--------|---------|----------------|
| Max memory (TPC-H SF=1) | >2GB (OOM) | <512MB |
| Buffer pool default | 128MB | 128MB |
| Query cache memory | 64MB | 64MB |
| WAL buffer | 16MB | 16MB |

---

## Coverage Targets

| Crate | Alpha | Beta | RC | GA |
|-------|-------|------|----|----|
| parser | 65% | 65% | 70% | **75%** |
| executor | 55% | 55% | 65% | **70%** |
| planner | 50% | 50% | 55% | **60%** |
| optimizer | 40% | 40% | 50% | **60%** |
| storage | 55% | 55% | 60% | **70%** |
| transaction | 50% | 50% | 60% | **70%** |
| catalog | 60% | 60% | 65% | **70%** |
| network | 45% | 45% | 55% | **60%** |
| mysql-server | 40% | 40% | 50% | **55%** |
| **Overall** | **50%** | **50%** | **60%** | **≥65%** |

---

## Formal Proofs

| Phase | Target | Count |
|-------|--------|-------|
| Alpha | ≥10 | ~30 (from v3.0.0) |
| Beta | ≥20 | ~30 |
| RC | ≥30 | ~35 |
| GA | **≥30** | ≥30 |

---

## Scalability

| Dimension | Target |
|-----------|--------|
| Max concurrent connections | 1000 |
| Max tables per database | 10000 |
| Max columns per table | 1017 |
| Max row size | 64KB |
| Max index per table | 64 |
| Max partitions per table | 8192 |
