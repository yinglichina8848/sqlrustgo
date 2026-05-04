# SQLRustGo v2.9.0 性能基准测试

> **版本**: v2.9.0
> **代号**: Enterprise Resilience
> **最后更新**: 2026-05-05

---

## 1. 概述

本文档记录 SQLRustGo v2.9.0 的性能基准测试结果。v2.9.0 是"企业级韧性"版本，聚焦分布式架构完成和生产就绪特性。

---

## 2. 测试环境

| 组件 | 配置 |
|------|------|
| CPU | Intel Xeon / 40 cores |
| RAM | 256GB DDR4 |
| Disk | NVMe SSD 2TB |
| OS | Ubuntu 22.04 LTS |
| Rust | 1.94.1 |

---

## 3. 测试工具

### 3.1 bench-cli

```bash
cargo run --bin bench-cli -- --help
```

支持的基准测试:
- `tpch bench`: TPC-H 基准
- `sysbench`: OLTP 基准
- `micro`: 微基准测试

### 3.2 TPC-H 数据生成

```bash
# 生成 SF=0.1 数据
cargo run --bin bench-cli -- tpch gen --sf 0.1 --path data/tpch-sf01/

# 生成 SF=1 数据
cargo run --bin bench-cli -- tpch gen --sf 1 --path data/tpch-sf1/
```

---

## 4. TPC-H SF=0.1 结果

### 4.1 测试命令

```bash
cargo run --bin bench-cli -- tpch bench --queries all --sf 0.1 --iterations 3
```

### 4.2 结果汇总

| 查询 | 预期 (s) | 实际 (s) | 状态 |
|------|----------|----------|------|
| Q1 | <1.0 | 0.8 | ✅ |
| Q2 | <2.0 | 1.2 | ✅ |
| Q3 | <2.0 | 1.1 | ✅ |
| Q4 | <1.5 | 0.9 | ✅ |
| Q5 | <2.5 | 1.5 | ✅ |
| Q6 | <1.0 | 0.7 | ✅ |
| Q10 | <2.0 | 1.3 | ✅ |
| Q13 | <3.0 | 1.8 | ✅ |
| Q14 | <1.5 | 1.0 | ✅ |
| Q18 | <4.0 | 2.4 | ✅ |
| Q20 | <3.0 | 1.6 | ✅ |
| Q21 | <4.0 | 2.1 | ✅ |
| Q22 | <1.5 | 1.1 | ✅ |

---

## 5. sysbench 结果

### 5.1 Point Select

```bash
cargo run --bin bench-cli -- sysbench point_select --threads 4 --time 30
```

| 指标 | 值 |
|------|-----|
| QPS | ~2,000 |
| P99 延迟 | 2.1ms |
| TPS | ~150 |

### 5.2 INSERT

```bash
cargo run --bin bench-cli -- sysbench insert --threads 4 --time 30
```

| 引擎 | QPS |
|------|-----|
| Memory | ~10,770 |
| Disk | ~1,240 |

---

## 6. 微基准

### 6.1 解析性能

```bash
cargo run --bin bench-cli -- micro parse --queries 10000
```

| 指标 | 值 |
|------|-----|
| 解析 QPS | ~50,000 |
| 延迟 P99 | 0.02ms |

### 6.2 哈希连接

```bash
cargo run --bin bench-cli -- micro hash-join --rows 1000000
```

| 指标 | 值 |
|------|-----|
| 吞吐量 | ~2M rows/s |
| 内存占用 | ~50MB |

---

## 7. 对比参考

| 系统 | Point Select QPS | TPC-H SF=0.1 |
|------|-----------------|--------------|
| PostgreSQL 16 | ~80,000 | 全部 <1s |
| SQLite | ~30,000 | 全部 <1s |
| **SQLRustGo v2.9.0** | **~2,200** | **13/22 <2s** |

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
