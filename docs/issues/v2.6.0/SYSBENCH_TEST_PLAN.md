# SQLRustGo Sysbench 测试计划 ISSUE

## 概述

本文档定义了 SQLRustGo 的 Sysbench 兼容性测试计划，使用开源 sysbench 工具对 SQLRustGo 进行全面基准测试。

## 背景

### 现有资产
- **bench crate**: 已有完整的 Rust 实现的 sysbench 兼容基准测试框架
  - 11 种 OLTP 工作负载 (oltp_point_select, oltp_read_only, oltp_read_write 等)
  - Uniform + Zipfian 分布模型
  - Warmup/Measurement/Cooldown 三阶段执行
  - 回归检测功能
- **设计规范**: `docs/superpowers/specs/2026-03-28-sysbench-compatible-benchmark.md`

### 目标
使用原生 sysbench 工具（而非内置 bench crate）测试 SQLRustGo，验证 MySQL 协议兼容性和性能基线。

## 测试策略

### 1. sysbench 工具准备

```bash
# 安装 sysbench (macOS)
brew install sysbench

# 或从源码编译
git clone https://github.com/akopytov/sysbench.git
cd sysbench
./autogen.sh
./configure
make -j
make install
```

### 2. SQLRustGo 启动

```bash
# 启动 SQLRustGo (MySQL 协议端口 3307)
cargo run --bin sqlrustgo -- --port 3307
```

### 3. 测试场景

#### 3.1 基本 OLTP 测试

| 测试场景 | 说明 | SQL 示例 |
|----------|------|----------|
| oltp_point_select | 主键点查 | SELECT * FROM sbtest1 WHERE id=? |
| oltp_read_only | 只读事务 | SELECT + JOIN + ORDER BY |
| oltp_read_write | 读写事务 | SELECT + UPDATE + INSERT |
| oltp_write_only | 只写事务 | UPDATE + DELETE + INSERT |
| oltp_mixed | 混合负载 | 读写混合 |
| oltp_insert | 插入测试 | BULK INSERT |
| oltp_delete | 删除测试 | DELETE + REPLACE |
| oltp_update_index | 索引更新 | UPDATE indexed columns |
| oltp_update_non_index | 非索引更新 | UPDATE non-indexed |
| oltp_range_scan | 范围扫描 | SELECT WHERE BETWEEN |
| oltp_index_scan | 索引扫描 | SELECT WHERE indexed |

#### 3.2 性能指标

| 指标 | 说明 | 目标 |
|------|------|------|
| QPS | 每秒查询数 | ≥ 1000 |
| TPS | 每秒事务数 | ≥ 100 |
| Latency P50 | P50 延迟 | < 50ms |
| Latency P95 | P95 延迟 | < 100ms |
| Latency P99 | P99 延迟 | < 200ms |
| Threads | 并发线程数 | 1, 10, 50, 100 |

#### 3.3 数据规模

| 规模 | 表数量 | 每表行数 | 说明 |
|------|--------|----------|------|
| SF1 | 1 | 100,000 | 小规模 |
| SF10 | 1 | 1,000,000 | 中规模 |
| SF50 | 4 | 1,000,000 | 大规模 |

### 4. 测试命令模板

```bash
# 1. 数据准备 (prepare)
sysbench oltp_read_write \
    --db-driver=mysql \
    --mysql-host=127.0.0.1 \
    --mysql-port=3307 \
    --mysql-user=root \
    --mysql-password= \
    --mysql-db=sbtest \
    --threads=50 \
    --tables=1 \
    --table-size=100000 \
    prepare

# 2. 运行测试 (run)
sysbench oltp_read_write \
    --db-driver=mysql \
    --mysql-host=127.0.0.1 \
    --mysql-port=3307 \
    --mysql-user=root \
    --mysql-password= \
    --mysql-db=sbtest \
    --threads=50 \
    --tables=1 \
    --table-size=100000 \
    --time=60 \
    --report-interval=10 \
    run

# 3. 清理数据 (cleanup)
sysbench oltp_read_write \
    --db-driver=mysql \
    --mysql-port=3307 \
    cleanup
```

## 实现任务

### Phase 1: 环境准备 (TBD)

- [ ] 安装 sysbench 工具
- [ ] 验证 SQLRustGo MySQL 协议兼容性
- [ ] 创建测试数据库和用户

### Phase 2: 基本测试 (TBD)

- [ ] oltp_point_select 测试
- [ ] oltp_read_only 测试
- [ ] oltp_read_write 测试
- [ ] 收集基线数据

### Phase 3: 扩展测试 (TBD)

- [ ] 所有 11 种工作负载测试
- [ ] 多规模数据测试 (SF1/SF10/SF50)
- [ ] 并发扩展测试 (1/10/50/100 threads)

### Phase 4: 性能调优 (TBD)

- [ ] 性能瓶颈分析
- [ ] 优化建议实施
- [ ] 再次测试验证

## 门禁检查清单

### 发布前必须通过

| # | 检查项 | 阈值 | 状态 |
|---|--------|------|------|
| 1 | oltp_point_select QPS | ≥ 1000 | ⬜ |
| 2 | oltp_read_only QPS | ≥ 800 | ⬜ |
| 3 | oltp_read_write QPS | ≥ 500 | ⬜ |
| 4 | P99 延迟 | < 200ms | ⬜ |
| 5 | 并发 50 线程稳定 | 无崩溃 | ⬜ |
| 6 | 数据一致性 | ACID 验证通过 | ⬜ |

## 对比基准

### 预期 vs MySQL

| 指标 | MySQL 8.0 | 预期 SQLRustGo | 比率 |
|------|-----------|----------------|------|
| QPS (point_select) | 50,000+ | 1,000+ | 2% |
| QPS (read_write) | 10,000+ | 500+ | 5% |
| Latency P99 | 10ms | 200ms | 20x |

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| MySQL 协议不完全兼容 | 测试无法运行 | 优先修复协议兼容性问题 |
| 性能差距过大 | 失去测试意义 | 调整目标阈值 |
| 数据不一致 | 测试结果不可信 | 增加一致性校验 |

## 时间线

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| Week 1 | 环境准备 + 基本测试 | 7 天 |
| Week 2 | 扩展测试 + 数据收集 | 7 天 |
| Week 3 | 性能调优 + 验证 | 7 天 |
| Week 4 | 门禁检查 + 报告 | 7 天 |

## 附件

- sysbench 官方文档: https://github.com/akopytov/sysbench
- SQLRustGo bench crate: `crates/bench/`
- 设计规范: `docs/superpowers/specs/2026-03-28-sysbench-compatible-benchmark.md`