# v3.0.0 Alpha 性能目标

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06

---

## 性能目标总览

| 指标 | v2.9.0 实际 | v3.0.0 Alpha 目标 | 提升 |
|------|-------------|------------------|------|
| Point SELECT QPS | ~2,000 | ≥20,000 | 10x |
| UPDATE QPS | ~950 | ≥10,000 | 10x |
| DELETE QPS | ~206 | ≥10,000 | 48x |
| Point SELECT (并发8T) | — | ≥5,000 | — |
| TPC-H SF=0.1 | 22/22 (~10.9s) | 22/22 可运行 | ✅ |
| TPC-H SF=1 | OOM | 22/22 无 OOM | — |

---

## QPS 基线

### 基准测试命令

```bash
cargo test --test qps_benchmark_test -- --ignored --nocapture
```

### E-09 性能地板（必须满足）

| 操作 | 最小 QPS | 说明 |
|------|----------|------|
| UPDATE | ≥10,000 | 硬性地板 |
| DELETE | ≥10,000 | 硬性地板 |

### QPS 目标

| 操作 | v2.9.0 | v3.0.0 Alpha 目标 |
|------|---------|-------------------|
| simple_select | ~2,000 | ≥20,000 |
| insert | — | ≥5,000 |
| update | ~950 | ≥10,000 |
| delete | ~206 | ≥10,000 |
| join | — | ≥8,000 |
| aggregation | — | ≥5,000 |
| order_by | — | ≥8,000 |
| concurrent_select_8t | — | ≥5,000 |
| complex_where | — | ≥5,000 |

---

## TPC-H 性能目标

### Alpha 阶段 (SF=0.1)

| 指标 | 目标 |
|------|------|
| 通过率 | 22/22 |
| 总耗时 | ≤15s |
| Q1 耗时 | ≤5s |
| Q6 耗时 | ≤3s |

### Beta 阶段 (SF=1)

| 指标 | 目标 |
|------|------|
| 通过率 | 22/22 |
| 无 OOM | 必须 |
| 总耗时 | ≤60s |
| p99 耗时 | ≤30s |

### GA 阶段 (SF=1)

| 指标 | 目标 |
|------|------|
| 通过率 | 22/22 |
| 总耗时 | ≤30s |
| p99 耗时 | ≤2s |

---

## Sysbench 性能目标

### Alpha 阶段

| 测试 | 目标 QPS |
|------|---------|
| point_select | ≥30,000 |
| oltp_read_write | ≥10,000 |
| oltp_write_only | ≥8,000 |
| update_index | ≥8,000 |

### Beta/GA 阶段

| 测试 | 目标 QPS |
|------|---------|
| point_select | ≥50,000 |
| oltp_read_write | ≥20,000 |
| oltp_write_only | ≥15,000 |
| update_index | ≥15,000 |

---

## 回归检测

### check_regression.sh

| 阈值 | 判定 |
|------|------|
| ≤5% 回归 | PASS（噪声范围内） |
| 5-20% 回归 | WARN（需 PR 说明） |
| >20% 回归 | FAIL（必须修复） |
| concurrent_select_8t | 放宽至 30% |

---

## 内存限制

| 场景 | 内存限制 |
|------|----------|
| 单次查询 | 8GB |
| TPC-H SF=1 | 无 OOM |
| HashJoin/Sort | 可配置限额 + 落盘（Beta） |