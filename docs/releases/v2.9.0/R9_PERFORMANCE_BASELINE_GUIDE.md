# R9 性能基准建立指南

> **版本**: v2.9.0
> **最后更新**: 2026-05-06 (E-09 后更新)
> **作者**: Claude Code / SQLRustGo Agent

---

## 一、概述

R9 是 R-Gate 系统中的性能基准检查门禁，确保每次提交不会导致显著的性能退化。

### R9 要求（来自 gate_spec.md）

```bash
# R9 命令
bash scripts/gate/check_regression.sh

# 基准文件位置
perf_baselines/v2.9.0/baseline.json

# 性能回归阈值
<=5% = PASS（通过）
5-20% = 需要解释
>20% = FAIL（不通过）
```

---

## 二、当前 SQLRustGo 性能基准（E-09 优化后）

### 2.1 QPS 基准结果（2026-05-06 实测，MemoryStorage）

| 操作 | 目标 QPS | 优化前 | **优化后 QPS** | 达成率 | 状态 |
|------|----------|--------|-------------|--------|------|
| Aggregation | - | 195,921 | **1,643,824** | - | 极佳 |
| ORDER BY | - | 53,539 | **81,988** | - | 良好 |
| **DELETE** | >=10,000 | 206 | **63,568** | 636% | ✅ 超标 6.4x |
| **JOIN** | >=10,000 | 12,617 | **57,388** | 574% | ✅ 超标 5.7x |
| **UPDATE** | >=10,000 | 950 | **43,224** | 432% | ✅ 超标 4.3x |
| **INSERT** | >=10,000 | 11,545 | **33,377** | 334% | ✅ 超标 3.3x |
| **Simple SELECT** | >=10,000 | 9,559 | **24,516** | 245% | ✅ 达标 |
| Concurrent SELECT (8t) | - | 7,620 | **11,995** | - | 良好 |
| Complex WHERE | - | 924 | **1,226** | - | 中等 |

**所有目标 QPS（>=10,000）的指标全部达标。**

### 2.2 E-09 优化原理

DELETE/UPDATE 在索引命中时使用**原位操作**：
- DELETE: `find_by_index` -> `delete_by_indices`（无 delete-all-reinsert）
- UPDATE: `find_by_index` -> `get_table_records_mut`（直接修改 Vec 中行）

非索引 WHERE 条件仍走 fallback 全扫描路径。

### 2.3 sysbench OLTP 基准（真实环境需验证）

| 线程数 | Point Select QPS | 延迟 P99 |
|--------|-----------------|---------|
| 1 | 520 | 1.8ms |
| 4 | 1,950 | 2.1ms |
| 8 | 2,100 | 3.8ms |
| 16 | 2,200 | 7.2ms |

**当前**: ~2,200 QPS（目标 >=10,000，延至 v2.10.0）

---

## 三、TPC-H 工作负载状态

### 3.1 重要发现

**SQLRustGo 尚未实现 TPC-H 工作负载。**

`bench-cli` 虽然接受 `--workload tpch` 参数，但 `crates/bench/src/workload/` 下只有 OLTP 工作负载。

### 3.2 当前可用的基准测试

```bash
# 内置 QPS 测试（推荐用于 R9）
cargo test --test qps_benchmark_test -- --ignored --nocapture

# OLTP 基准（需要 SQLRustGo TCP 服务在 4000 端口运行）
cargo run -p sqlrustgo-bench -- \
  --db sqlrustgo \
  --workload oltp \
  --threads 4 \
  --duration 10 \
  --scale 5000
```

### 3.3 TPC-H 需求

若要运行 TPC-H 基准，需要：

1. **实现 TPC-H 工作负载**（22 条 TPC-H SQL 查询）
2. **数据生成器**（TPC-H 规范要求的 dbgen）
3. **查询模板**（Q14 等需要视图支持）

---

## 四、R9 门禁使用指南

### 4.1 基线文件

基线已建立于 `perf_baselines/v2.9.0/baseline.json`（2026-05-06），包含 9 个 QPS 基准指标。

### 4.2 运行 R9 回归检测

```bash
# 完整运行（跑所有基准 + 对比基线）— 约 5-10 分钟
bash scripts/gate/check_regression.sh

# 跳过基准运行（使用已有的 current.json）
bash scripts/gate/check_regression.sh --skip-run
```

### 4.3 更新基线

当确认性能提升并希望设定新基线时：
```bash
# 1. 先运行 check_regression.sh 生成 current.json
bash scripts/gate/check_regression.sh --skip-run

# 2. 手动更新 baseline.json 中各 benchmark 的 qps 值
# 3. 提交 baseline.json 作为新的性能基线
```

### 4.4 解读结果

```
<=5% 衰减  = ✅ PASS（噪音范围）
5-20% 衰减 = ⚠️  WARN（需在 PR 描述中解释原因）
>20% 衰减 = ❌ FAIL（要求修复后重新提交）
```

---

## 五、E-09 优化记录

### 5.1 已完成的优化

| 方案 | PR | 内容 | 效果 |
|------|-----|------|------|
| 方案 4 | #313 | 表达式缓存（`evaluate_sql_expression` 缓存） | 全局 1.5-2x |
| 方案 2 | #317, #322 | 消除双重扫描 + AST 评估 | 全局 2-3x |
| 方案 1（部分） | 同上 | 索引命中时原位 delete/update | UPDATE/DELETE 10-30x |

### 5.2 已知局限

- 非索引 WHERE 条件仍走 fallback 全扫描+重建路径
- FileStorage 的 `delete_by_indices` 仅 MemoryStorage 充分测试
- Complex WHERE (LIKE) 因无倒排索引较慢 (1,226 QPS)
- 方案 1 未完全实现（通用原位操作仅索引命中时启用）

### 5.3 后续优化（v3.0.0）

- Phase 1 Performance Pocket: CBO 完善、连接池、Group Commit
- 预期进一步提升 3-10x 吞吐

---

## 六、给 Hermes 的建议

### 6.1 R9 已就绪

- ✅ 基线文件: `perf_baselines/v2.9.0/baseline.json`
- ✅ 检测脚本: `scripts/gate/check_regression.sh`
- ✅ QPS 基准: 9 项指标全部可用
- ✅ E-09 优化: DELETE 63K / UPDATE 43K / 全部达标

### 6.2 下一步

1. **集成到 CI Gate**: 在 Hermes Pipeline 中加入 `bash scripts/gate/check_regression.sh --skip-run`
2. **TPC-H 工作负载**: 实现后纳入 R9
3. **真实 sysbench**: TCP 模式下的 OLTP 测试

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [E08_QPS_BENCHMARK_REPORT.md](./E08_QPS_BENCHMARK_REPORT.md) | E-08/E-09 QPS 详细测试结果（已更新） |
| [E09_UPDATE_DELETE_QPS_OPTIMIZATION_PLAN.md](./E09_UPDATE_DELETE_QPS_OPTIMIZATION_PLAN.md) | E-09 优化计划 |
| [PERFORMANCE_REPORT.md](./PERFORMANCE_REPORT.md) | v2.9.0 性能报告 |
| [BENCHMARK.md](./BENCHMARK.md) | 基准测试指南 |

---

## 八、更新日志

| 日期 | 作者 | 更新内容 |
|------|------|----------|
| 2026-05-06 | Claude Code | 初始版本 |
| 2026-05-06 | SQLRustGo Agent | E-09 后更新：实测 QPS 数据、R9 脚本就绪、去除过时信息 |

---

*本文档由 SQLRustGo Team 维护*