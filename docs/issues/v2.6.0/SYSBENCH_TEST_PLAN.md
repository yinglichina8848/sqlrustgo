# SQLRustGo Sysbench 测试计划 ISSUE

## 概述

本文档定义了 SQLRustGo 的 Sysbench 兼容性测试计划，使用开源 sysbench 工具对 SQLRustGo 进行全面基准测试。

## 背景

### 目标
使用原生 sysbench 工具（而非内置 bench crate）测试 SQLRustGo，验证 MySQL 协议兼容性和性能基线。

## 测试策略

### 1. sysbench 工具准备

```bash
# 安装 sysbench (macOS)
brew install sysbench
```

### 2. SQLRustGo 启动

```bash
# 启动 SQLRustGo (MySQL 协议端口 3307)
cargo run --bin sqlrustgo -- --port 3307
```

### 3. 测试场景

#### 3.1 基本 OLTP 测试

| 测试场景 | 说明 |
|----------|------|
| oltp_point_select | 主键点查 |
| oltp_read_only | 只读事务 |
| oltp_read_write | 读写事务 |
| oltp_write_only | 只写事务 |
| oltp_mixed | 混合负载 |
| oltp_insert | 插入测试 |
| oltp_delete | 删除测试 |
| oltp_update_index | 索引更新 |
| oltp_update_non_index | 非索引更新 |
| oltp_range_scan | 范围扫描 |
| oltp_index_scan | 索引扫描 |

### 4. 门禁检查清单

| # | 检查项 | 阈值 |
|---|--------|------|
| 1 | oltp_point_select QPS | ≥ 1000 |
| 2 | oltp_read_only QPS | ≥ 800 |
| 3 | oltp_read_write QPS | ≥ 500 |
| 4 | P99 延迟 | < 200ms |
| 5 | 并发 50 线程稳定 | 无崩溃 |
| 6 | 数据一致性 | ACID 验证通过 |

## 相关文档

- 设计规范: `docs/superpowers/specs/2026-03-28-sysbench-compatible-benchmark.md`
- bench crate: `crates/bench/`