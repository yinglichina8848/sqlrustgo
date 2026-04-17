# v2.6.0 性能目标

> **版本**: v2.6.0
> **创建日期**: 2026-04-17
> **维护人**: yinglichina8848

---

## 一、OLTP 性能目标

### 1.1 点查性能

| 指标 | v2.5.0 | v2.6.0 目标 | 提升 |
|------|---------|--------------|------|
| TPS (32 并发) | 50,000 | 75,000 | +50% |
| 延迟 P50 | - | < 1ms | - |
| 延迟 P99 | - | < 5ms | - |

### 1.2 索引扫描性能

| 指标 | v2.5.0 | v2.6.0 目标 | 提升 |
|------|---------|--------------|------|
| TPS (32 并发) | 10,000 | 15,000 | +50% |
| 延迟 P50 | - | < 10ms | - |
| 延迟 P99 | - | < 50ms | - |

### 1.3 插入性能

| 指标 | v2.5.0 | v2.6.0 目标 | 提升 |
|------|---------|--------------|------|
| TPS (16 并发) | 20,000 | 30,000 | +50% |
| 延迟 P50 | - | < 2ms | - |
| 延迟 P99 | - | < 10ms | - |

---

## 二、TPC-H 性能目标

### 2.1 SF=1 性能

| 查询 | v2.5.0 | v2.6.0 目标 | 提升 |
|------|---------|--------------|------|
| Q1 | ~320ms | < 200ms | +37% |
| Q2 | - | < 100ms | - |
| Q3 | - | < 150ms | - |
| ... | - | - | - |
| All Q | ~8.5s | < 5s | +41% |

### 2.2 SF=10 性能

| 查询 | v2.5.0 | v2.6.0 目标 |
|------|---------|--------------|
| Q1 | - | < 2s |
| All Q | - | < 50s |

---

## 三、并发性能目标

### 3.1 并发写入

| 指标 | v2.5.0 | v2.6.0 目标 |
|------|---------|--------------|
| 32+ 并发写入 | - | 正确性 100% |
| 死锁检测 | - | 自动恢复 |
| write-write 冲突 | - | 正确处理 |

### 3.2 隔离级别

| 隔离级别 | v2.5.0 | v2.6.0 目标 |
|----------|---------|--------------|
| Read Committed | ✅ | ✅ |
| Snapshot Isolation | ✅ | ✅ |
| Serializable (SSI) | ❌ | ✅ |

---

## 四、测试命令

### 4.1 OLTP 性能测试

```bash
# 点查基准测试
cargo bench --bench point_select_benchmark

# 索引扫描基准测试
cargo bench --bench index_scan_benchmark

# 插入基准测试
cargo bench --bench insert_benchmark
```

### 4.2 TPC-H 测试

```bash
# SF=1 完整测试
cargo test --test tpch_full_test -- --sf 1

# SF=10 性能测试
cargo test --test tpch_sf10_benchmark
```

### 4.3 并发压力测试

```bash
# 并发压力测试
cargo test --test concurrency_stress_test

# 死锁检测测试
cargo test --test deadlock_detection_test

# 隔离级别测试
cargo test mvcc_snapshot_isolation
cargo test mvcc_serializable
```

---

## 五、验收标准

| 测试 | 目标 | 命令 |
|------|------|------|
| TPC-H SF=1 | < 5s | `cargo test --test tpch_full_test -- --sf 1` |
| 并发压力测试 | 100% 通过 | `cargo test --test concurrency_stress_test` |
| MVCC SSI | 100% 通过 | `cargo test mvcc_serializable` |

---

## 六、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
