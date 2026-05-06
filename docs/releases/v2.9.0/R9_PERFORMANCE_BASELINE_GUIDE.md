# R9 性能基准建立指南

> **版本**: v2.9.0
> **最后更新**: 2026-05-06
> **作者**: Claude Code

---

## 一、概述

R9 是 R-Gate 系统中的性能基准检查门禁，确保每次提交不会导致显著的性能退化。

### R9 要求（来自 gate_spec.md）

```bash
# R9 命令
cargo bench && scripts/gate/check_regression.sh

# 基准文件位置
perf_baselines/v2.9.0/baseline.json

# 性能回归阈值
≤5% = PASS（通过）
5-20% = 需要解释
>20% = FAIL（不通过）
```

---

## 二、当前 SQLRustGo 性能基准

### 2.1 E-08 QPS 基准结果（已建立）

| 操作 | 目标 QPS | 实际 QPS | 达成率 | 状态 |
|------|----------|----------|--------|------|
| Aggregation | - | 195,921 | - | 极佳 |
| ORDER BY | - | 53,539 | - | 良好 |
| **JOIN** | ≥10,000 | 12,617 | 126% | ✅ |
| **INSERT** | ≥10,000 | 11,545 | 115% | ✅ |
| Simple SELECT | ≥10,000 | 9,559 | 96% | ⚠️ |
| **UPDATE** | ≥10,000 | 950 | 9.5% | ❌ |
| **DELETE** | ≥10,000 | 206 | 2% | ❌ |

### 2.2 sysbench OLTP 基准

| 线程数 | Point Select QPS | 延迟 P99 |
|--------|-----------------|---------|
| 1 | 520 | 1.8ms |
| 4 | 1,950 | 2.1ms |
| 8 | 2,100 | 3.8ms |
| 16 | 2,200 | 7.2ms |

**当前**: ~2,200 QPS（目标 ≥10,000，延至 v2.10.0）

### 2.3 INSERT 基准

| 引擎 | QPS | 延迟 P99 |
|------|-----|---------|
| MemoryExecutionEngine | 10,770 | 0.37ms |
| DiskExecutionEngine | 1,240 | 3.2ms |

---

## 三、TPC-H 工作负载状态

### 3.1 重要发现

**SQLRustGo 尚未实现 TPC-H 工作负载。**

`bench-cli` 虽然接受 `--workload tpch` 参数，但 `crates/bench/src/workload/` 下只有 OLTP 工作负载：

```
oltp.rs, oltp_delete.rs, oltp_index_scan.rs, oltp_insert.rs,
oltp_mixed.rs, oltp_point_select.rs, oltp_range_scan.rs,
oltp_read_only.rs, oltp_read_write.rs, oltp_update_index.rs,
oltp_update_non_index.rs, oltp_write_only.rs
```

**没有 TPC-H 工作负载实现。**

### 3.2 当前可用的基准测试

```bash
# OLTP 基准（需要 SQLRustGo TCP 服务在 4000 端口运行）
cargo run -p sqlrustgo-bench -- \
  --db sqlrustgo \
  --workload oltp \
  --threads 4 \
  --duration 10 \
  --scale 5000

# 内置 QPS 测试（通过 cargo test）
cargo test --test qps_benchmark_test -- --ignored
```

### 3.3 TPC-H 需求

若要运行 TPC-H 基准，需要：

1. **实现 TPC-H 工作负载**（22 条 TPC-H SQL 查询）
2. **数据生成器**（TPC-H 规范要求的 dbgen）
3. **查询模板**（Q14 等需要视图支持）

---

## 四、建立 R9 性能基准的步骤

### 4.1 步骤 1：建立 OLTP 基准

```bash
# 1. 运行 OLTP 基准测试
cargo run -p sqlrustgo-bench -- \
  --db sqlrustgo \
  --workload oltp \
  --threads 4 \
  --duration 60 \
  --scale 5000 \
  --report-json results/oltp_baseline.json

# 2. 保存基准
mkdir -p perf_baselines/v2.9.0
cp results/oltp_baseline.json perf_baselines/v2.9.0/baseline.json
```

### 4.2 步骤 2：运行 QPS 基准测试

```bash
# 运行 QPS 基准
cargo test --test qps_benchmark_test -- --ignored 2>&1 | tee results/qps_baseline.txt

# 从输出中提取 QPS 数据，手动记录到 baseline.json
```

### 4.3 步骤 3：配置 check_regression.sh

```bash
# 脚本位置
scripts/gate/check_regression.sh

# 需要的配置
THRESHOLD_PCT=5
BASELINE_FILE=perf_baselines/v2.9.0/baseline.json
```

### 4.4 步骤 4：验证 R9 门禁

```bash
# 手动运行 R9 检查
cargo bench 2>&1 | tee results/bench_current.txt
bash scripts/gate/check_regression.sh
```

---

## 五、E-08 QPS 测试改进建议

### 5.1 当前问题

根据 E-08 QPS 基准测试，UPDATE/DELETE QPS 严重不达标：

| 操作 | 目标 QPS | 实际 QPS | 问题 |
|------|----------|----------|------|
| UPDATE | ≥10,000 | 950 | scan-filter-delete-all-reinsert 反模式 |
| DELETE | ≥10,000 | 206 | 三倍表扫描 |

### 5.2 根因分析

当前实现的反模式：

```
1. 找匹配行 → 全表扫描
2. 找保留行 → 全表扫描
3. 删除所有行 → 全表扫描
4. 重建保留行 → 全表扫描
5. 获取锁 6-7 次
```

### 5.3 解决方案（E-09）

使用 StorageEngine API 的 `update/delete` 方法：

```rust
// 正确做法：直接调用 StorageEngine 的 delete 方法
fn execute_delete(&self, table: &str, predicate: &Expr) -> SqlResult<usize> {
    self.storage.delete(table, predicate)?  // 一次扫描
}
```

---

## 六、给 Hermes 的建议

### 6.1 R9 基准建立优先级

1. **建立 OLTP QPS 基准**（当前可做）
   - 使用 `bench-cli --workload oltp`
   - 记录 Point Select、Range Scan、INSERT 等指标

2. **建立 DELETE QPS 基准**（当前可做）
   - 使用 `cargo test --test qps_benchmark_test -- --ignored`
   - 关注 DELETE QPS（当前 206）

3. **TPC-H 工作负载**（需要实现）
   - 告知用户当前不支持
   - 建议先完成 E-09 UPDATE/DELETE 优化

### 6.2 外部工具方案

若需要立即进行 TPC-H 测试，可使用外部工具：

```bash
# 使用 PostgreSQL TPC-H
psql -h localhost -U postgres -d tpch -f queries/q1.sql

# 使用 DuckDB
duckdb -c "SELECT * FROM lineitem LIMIT 10;"
```

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [E08_QPS_BENCHMARK_REPORT.md](./E08_QPS_BENCHMARK_REPORT.md) | E-08 QPS 详细测试结果 |
| [PERFORMANCE_REPORT.md](./PERFORMANCE_REPORT.md) | v2.9.0 性能报告 |
| [BENCHMARK.md](./BENCHMARK.md) | 基准测试指南 |
| [E09_UPDATE_DELETE_QPS_OPTIMIZATION_PLAN.md](./E09_UPDATE_DELETE_QPS_OPTIMIZATION_PLAN.md) | E-09 优化计划 |

---

## 八、更新日志

| 日期 | 作者 | 更新内容 |
|------|------|----------|
| 2026-05-06 | Claude Code | 初始版本，添加 TPC-H 工作负载状态说明 |

---

*本文档由 Claude Code 创建*
