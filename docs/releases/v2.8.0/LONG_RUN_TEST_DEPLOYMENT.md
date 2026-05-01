# v2.8.0 长稳测试部署指南

> **版本**: v2.8.0 (GA)
> **日期**: 2026-05-02
> **基于**: `tests/long_run_stability_test.rs` + `tests/long_run_stability_72h_test.rs`
> **测试总数**: 14 (均标记 `#[ignore]`)

---

## 1. 执行摘要

SQLRustGo v2.8.0 包含 **14 个长时间运行稳定性测试**，分为两组：

| 测试文件 | 测试数 | 模拟时长 | 存储引擎 |
|----------|--------|----------|----------|
| `long_run_stability_test.rs` | 10 | 加速模拟 72h | MemoryStorage |
| `long_run_stability_72h_test.rs` | 4 | 实际 72h 运行 | FileStorage + MemoryStorage |

**所有 14 个测试默认标记 `#[ignore]`**，需通过 `--ignored` 标志显式运行。
这些测试不纳入 CI，仅用于夜间/发布前验证。

---

## 2. 核心参数

```rust
// 两组测试共享的核心常量
const STABILITY_ITERATIONS: usize = 1000;   // 每次测试迭代次数
const CONCURRENT_THREADS: usize = 8;        // 并发线程数
```

| 参数 | 值 | 说明 |
|------|-----|------|
| 单测试迭代次数 | 1000 | 模拟持续压力 |
| 并发线程数 | 8 | 与默认 CPU 核心数匹配 |
| 测试数据量 | 10K-100K 行 | 每场景不同 |
| 72h 测试持续时间 | 259200 秒 | 实际运行 72 小时 |
| 监控间隔 | 600 秒 | 每 10 分钟记录系统状态 |
| 日志目录 | `test_results_72h/` | 72h 测试输出目录 |

---

## 3. 部署与运行

### 3.1 运行加速模拟测试（推荐，约 30-60 秒）

```bash
# 运行全部 10 个加速长稳测试
cargo test --test long_run_stability_test -- --ignored

# 运行单个测试
cargo test --test long_run_stability_test test_sustained_write_load -- --ignored
```

### 3.2 运行实际 72h 测试（仅发布前执行）

```bash
# 需要 release 模式以加速
cargo test --test long_run_stability_72h_test --release -- --ignored

# 运行单个 72h 测试
cargo test --test long_run_stability_72h_test test_sustained_write_72h --release -- --ignored
```

### 3.3 资源要求

| 资源 | 加速模拟 | 实际 72h |
|------|----------|----------|
| 运行时内存 | ~256 MB | ~512 MB |
| 磁盘空间 | - | ≥ 10 GB (WAL+数据) |
| 运行时长 | 30-60 秒 | 72 小时 |
| CPU 占用 | 8 线程 | 8 线程 |
| 日志收集 | 无需 | 自动写入 `test_results_72h/` |

---

## 4. 加速模拟测试详情（10 个测试）

### 4.1 `test_sustained_write_load` — 持续写入负载

- **场景**: 模拟 72h 持续写入
- **迭代**: 1000 次 INSERT
- **验证**: 所有插入成功 + COUNT 验证
- **输出**: ops/sec 吞吐量

### 4.2 `test_sustained_read_load` — 持续读取负载

- **场景**: 模拟 72h 持续读取
- **前置数据**: 100 行
- **迭代**: 1000 次全表扫描
- **验证**: 所有扫描成功

### 4.3 `test_concurrent_read_write_stability` — 并发读写稳定性

- **场景**: 8 个写入线程 + 8 个读取线程
- **每线程迭代**: 100 次
- **验证**: 0 错误
- **统计**: 记录写入数、读取数、错误数

### 4.4 `test_repeated_create_drop_stability` — 重复创建/删除

- **场景**: 100 次 CREATE TABLE + DROP TABLE
- **验证**: 每次创建/删除成功
- **输出**: ops/sec 吞吐量

### 4.5 `test_memory_stability_under_load` — 内存负载稳定性

- **场景**: 50 个并发批次 × 每批 10 次 = 5000 次插入
- **验证**: 无死锁、无 panic
- **线程数**: 50 个并发线程

### 4.6 `test_table_info_consistency_under_load` — 表元数据一致性

- **场景**: 1000 次 CREATE → SHOW TABLES → DROP
- **迭代**: 1000 次（3000 次操作）
- **验证**: 元数据操作无异常

### 4.7 `test_list_tables_stability` — 表列表稳定性

- **前置**: 创建 50 张表
- **迭代**: 1000 次 SHOW TABLES
- **验证**: 列表操作稳定性

### 4.8 `test_interleaved_read_write_consistency` — 交错读写一致性

- **场景**: 10 个线程 × 100 次（每个线程：INSERT + SELECT）
- **验证**: 读写交替不导致不一致

### 4.9 `test_rapid_burst_writes` — 突发写入

- **场景**: 10 个突发 × 100 并行插入 = 1000 次插入
- **验证**: 突发写入后数据完整性
- **输出**: 突发写入吞吐量

### 4.10 `test_stress_table_operations` — 表操作压力测试

- **场景**: 创建 20 张表，每张表插入 50 行
- **总操作**: 20 × 51 ≈ 1020 次操作
- **验证**: COUNT(*) 查询全部通过
- **清理**: 自动删除所有测试表

---

## 5. 实际 72h 测试详情（4 个测试）

### 5.1 `test_sustained_write_72h` — 持续写入 72 小时

| 参数 | 值 |
|------|-----|
| 存储引擎 | FileStorage |
| 循环缓冲区 | 100,000 行 |
| 监控间隔 | 600 秒 |
| 日志 | `test_results_72h/72h_test_progress.log` |

**行为**: 持续写入直到 72h 结束。每满 100K 行触发循环缓冲区清理（DROP + CREATE）。

### 5.2 `test_sustained_write_concurrent_72h` — 并发写入 72 小时

| 参数 | 值 |
|------|-----|
| 存储引擎 | MemoryStorage |
| 并发线程 | 8 |
| 监控 | 系统资源监控（RSS/CPU/Disk） |

**行为**: 8 个线程并发写入，统计总插入量和吞吐量。

### 5.3 `test_sustained_read_72h` — 持续读取 72 小时

| 参数 | 值 |
|------|-----|
| 存储引擎 | MemoryStorage |
| 前置数据 | 10,000 行 |
| 并发线程 | 8 |
| 查询模式 | `SELECT * FROM stability_test WHERE id % 100 = 0` |

**行为**: 8 个线程并发执行条件查询。

### 5.4 `test_concurrent_read_write_72h` — 混合读写 72 小时

| 参数 | 值 |
|------|-----|
| 存储引擎 | MemoryStorage |
| 写入线程 | 1 |
| 读取线程 | 8 |
| 数据模式 | 持续 INSERT + 条件 SELECT |

**行为**: 1 个写入线程 + 8 个读取线程并发，模拟真实 OLTP 混合负载。

---

## 6. 系统监控

72h 测试内置系统监控功能（`spawn_monitor_thread`）：

```rust
// 监控指标
fn log_system_stats(test_name, ops_count, elapsed_secs, remaining_secs) {
    // - RSS 内存 (MB)
    // - CPU 使用率 (%)
    // - 磁盘使用量 (MB)
    // - 操作计数
    // - 已用/剩余时间
}
```

每 10 分钟记录一次系统状态到日志文件。

**日志格式**:
```
STATS|ops=10000|elapsed=600s|remaining=258000s|RSS=128MB|cpu=45.2%|disk=256MB
```

---

## 7. 预期结果与验证标准

| 测试类别 | 预期结果 | 判断标准 |
|----------|----------|----------|
| 加速模拟（10 个） | 全部 ✅ PASS | 0 panic, 0 断言失败 |
| 实际 72h（4 个） | 无内存泄漏 | RSS 无明显增长 |
| 实际 72h（4 个） | 无 OOM/kill | 进程存活至结束 |
| 实际 72h（4 个） | 无数据损坏 | 日志无 IO 错误 |
| 全部 | ops/sec 稳定 | 吞吐量不持续下降 |

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 实际 72h 测试阻塞 CI | 发布流程延迟 | 仅发布前执行 |
| #[ignore] 测试被遗忘 | 稳定性检验缺失 | 纳入发布门禁清单 |
| FileStorage 磁盘满 | 测试失败 | 循环缓冲区自动清理 |
| 资源被回收 (OOM) | 测试中断 | MemoryStorage 单次 ≤ 512MB |

---

## 9. 与 v2.7.0 对比

| 维度 | v2.7.0 | v2.8.0 | 变化 |
|------|--------|--------|------|
| 加速测试数 | 6 | 10 | +4 (新增内存/元数据/突发/压力) |
| 实际 72h 测试数 | 0 | 4 | **新增** |
| FileStorage 支持 | ❌ | ✅ | 持久化存储测试 |
| 系统监控 | ❌ | ✅ | RSS/CPU/Disk 实时监控 |
| 日志输出 | ❌ | ✅ | 自动写入 `test_results_72h/` |
| 循环缓冲区 | ❌ | ✅ | 防止磁盘占满 |

---

## 10. 相关配置

```bash
# Cargo.toml 依赖
# 确保以下 crate 已启用:
sqlrustgo-storage = { path = "crates/storage" }
# 72h 测试依赖 FileStorage

# Rust 版本要求
# rustc 1.94.1+
# edition 2021
```

---

## 参考链接

- [稳定性测试报告](./STABILITY_REPORT.md)
- [功能矩阵](./FEATURE_MATRIX.md)
- [集成测试计划](./INTEGRATION_TEST_PLAN.md)
- [发布门禁清单](./RELEASE_GATE_CHECKLIST.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-02*
