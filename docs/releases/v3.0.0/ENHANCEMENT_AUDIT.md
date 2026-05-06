# v3.0.0 增强建议审计与任务补充

> **日期**: 2026-05-06
> **基于**: develop/v3.0.0 @ 819fc55c
> **审计目标**: 评估 6 条增强建议，定义可执行的任务

---

## 审计方法

每条建议经 **源码核查** 确认当前实现状态，然后给出：
- **状态**: ✅ 已实现 / ⚠️ 部分实现 / ❌ 未实现
- **差距**: 与生产需求的差距
- **建议**: 新增任务定义（含工时、文件、验收标准）

---

## 1. 网络层与协议优化

### 1.1 Prepared Statement 二进制协议

| 项目 | 结果 |
|------|------|
| 源码核查 | `COM_STMT_PREPARE` (line 1604) ✅ `COM_STMT_EXECUTE` (line 1722) ✅ `PreparedStatementManager` ✅ |
| 参数绑定 | `replace_placeholders()` ✅ 已实现 |
| 差距 | 参数缓存可能不完整：`replace_placeholders` 逐个替换 `?` 占位符，但 `params` 可能是空 Vec（line 1735） |
| **状态** | **⚠️ 已实现但参数传递路径可能有 bug** |

### 1.2 零拷贝读写缓冲区

| 项目 | 结果 |
|------|------|
| 当前实现 | `Vec<u8>` + `.write_to(stream)` — 每次分配新缓冲区 |
| 差距 | 无 `bytes` crate 的 `BytesMut` 零拷贝重用 |
| **状态** | **❌ 未实现** |

### 1.3 多语句执行 COM_MULTI

| 项目 | 结果 |
|------|------|
| 源码核查 | 无 `0x11` 常量，无处理分支 |
| **状态** | **❌ 未实现** |

### 新增任务

```
I-09: COM_MULTI 多语句执行
  文件: crates/mysql-server/src/lib.rs
  工时: 2d
  验收: sysbench oltp_read_write 可运行（prepare阶段需要）

I-10: Prepared Statement 参数绑定修复
  文件: crates/mysql-server/src/lib.rs (replace_placeholders)
  工时: 1d
  验收: COM_STMT_EXECUTE 带参数能正确替换

A-08: 零拷贝协议缓冲区（优化）
  文件: crates/mysql-server/src/protocol.rs
  工时: 3d (P2 可延后)
  验收: 点查询 100 并发无内存分配瓶颈
```

---

## 2. 存储引擎：聚簇索引基础实现

| 项目 | 结果 |
|------|------|
| 当前 B+Tree | 主键索引和数据分离存储 |
| ADR 产出 | D-05 仅产出架构决策文档（已标记 v3.1.0） |
| **状态** | **❌ 未实现** |

### 新增任务

```
S-01: 聚簇索引基础实现
  文件: crates/storage/src/bplus_tree/
  工时: 5d
  验收:
    - 主键检索时数据直接从索引叶节点读取（减少一次随机 I/O）
    - 非主键索引仍走旧路径
  注意: 仅针对 MemoryStorage 验证，FileStorage 延后
```

---

## 3. Optimizer/Planner 单元测试

| 项目 | 结果 |
|------|------|
| 当前状态 | Phase 1 新增 4 个桥接测试（86→86 tests） |
| optimizer 模块 | 之前覆盖率报告中约 55%（目标 70%） |
| planner 模块 | 之前覆盖率报告 <1%（目标 80%） |
| **状态** | **⚠️ 大幅改进但未达目标** |

### 新增任务

```
F-06t: Optimizer 规则测试扩展
  文件: crates/optimizer/tests/
  工时: 2d
  验收: optimizer 覆盖率 ≥70%

F-06p: Planner 逻辑测试扩展
  文件: crates/planner/tests/
  工时: 2d
  验收: planner 覆盖率 ≥80%
```

---

## 4. 压测与实践修复（Sysbench/事务）

### 4.1 "No transaction in progress" 错误

| 项目 | 结果 |
|------|------|
| 源码核查 | 事务状态机 `current_tx_id: Option<TxId>` 可能在某些路径下未正确初始化为事务 |
| **状态** | **⚠️ 依赖具体场景** |

### 4.2 隔离级别状态机

| 项目 | 结果 |
|------|------|
| 当前 | RC/SI ✅，但 SERIALIZABLE 尚未验收（F-04 标记为未完成） |
| **状态** | **⚠️ SERIALIZABLE 未完整验证** |

### 新增任务

```
T-01: Sysbench OLTP 完整适配
  文件: tests/e2e/
  工时: 3d
  验收:
    - oltp_read_only 通过
    - oltp_write_only 通过
    - oltp_read_write 通过
    - 所有场景 QPS 可测量（即使较低）

T-02: 事务状态机压力测试
  文件: crates/transaction/tests/
  工时: 2d
  验收:
    - 100 并发 BEGIN/COMMIT/ROLLBACK 无状态泄漏
    - 嵌套事务正确回滚
```

---

## 5. 内存治理（TPC-H SF=1 OOM 防护）

| 项目 | 结果 |
|------|------|
| 当前 | TPC-H SF=1 有 OOM 风险 |
| Hash Join 内存 | 无中间结果释放机制 |
| Sort | 全内存排序 |
| **状态** | **❌ 未实现** |

### 新增任务

```
M-01: Hash Join/Sort 内存限额
  文件: crates/executor/src/local_executor.rs
  工时: 3d
  验收:
    - 配置项 max_memory_per_query (默认 512MB)
    - 超限时使用临时文件 spill（Sort 可做，Hash Join 标记为 fallback）
    - TPC-H SF=1 22/22 可运行无 OOM

M-02: TPC-H SF=1 持续验证
  文件: scripts/gate/check_tpch.sh
  工时: 1d
  验收: SF=1 CI gate 可运行（允许 p99 < 30s 初始阈值）
```

---

## 6. 运维可观测性增强

### 6.1 Performance Schema 轻量版

| 项目 | 结果 |
|------|------|
| 当前 | 无 `events_statements_summary_by_digest` |
| `/metrics` | 存在 `MetricsRegistry` 但指标种类有限 |
| **状态** | **⚠️ 基础存在但不够细** |

### 新增任务

```
I-09: 性能摘要表
  文件: crates/information-schema/src/
  工时: 2d
  验收:
    - INFORMATION_SCHEMA.EVENTS_STATEMENTS_SUMMARY 可查询
    - 按 digest 聚合执行次数、总耗时、平均耗时

M-03: /metrics 增强
  文件: crates/server/src/metrics_endpoint.rs
  工时: 1d
  验收:
    - 新增指标: 连接数、缓存命中率、CBO 统计、查询延迟 P50/P95/P99
    - 提供 Grafana dashboard JSON
```

---

## 综合优先级与任务表

| 优先级 | # | 任务 | 工时 | 依赖 | 阶段 |
|--------|---|------|------|------|------|
| **P0** | M-01 | Hash Join/Sort 内存限额 | 3d | 无 | Phase 1 末 |
| **P0** | T-01 | Sysbench OLTP 适配 | 3d | I-09 | Phase 3 |
| **P0** | T-02 | 事务状态机压力测试 | 2d | 无 | Phase 2 |
| **P0** | I-09-ps | COM_MULTI 多语句 | 2d | 无 | Phase 3 |
| **P0** | I-10 | Prepared Statement 绑定修复 | 1d | 无 | Phase 3 |
| **P1** | F-06t | Optimizer 测试扩展 | 2d | 无 | Phase 2 |
| **P1** | F-06p | Planner 测试扩展 | 2d | 无 | Phase 2 |
| **P1** | M-02 | TPC-H SF=1 CI gate | 1d | M-01 | Phase 1 末 |
| **P2** | S-01 | 聚簇索引基础 | 5d | 架构评估 | Phase 0 或 v3.1 |
| **P2** | A-08 | 零拷贝缓冲区 | 3d | 无 | Phase 4 |
| **P2** | I-09-m | /metrics 增强 | 1d | 无 | Phase 3 |
| **P2** | I-09-sum | 性能摘要表 | 2d | 无 | Phase 3 |

### 对 6 条建议的最终评估

| 建议 | 评估 | 建议处理 |
|------|------|---------|
| 1. 协议深度优化 | ⚠️ Prepared Statement 已实现（参数绑定需修），COM_MULTI 缺，零拷贝 P2 | P0 修绑定 + COM_MULTI |
| 2. 聚簇索引 | ❌ 未实现，工期长（5d） | P2 → v3.1.0（不改计划） |
| 3. 单元测试补全 | ⚠️ 已加 4 个测试，仍需补 optimizer/planner | P1 Phase 2 整合 |
| 4. Sysbench/事务 | ⚠️ COM_MULTI 缺 + 事务状态机可能脆弱 | P0 Phase 3 修复 |
| 5. 内存治理 | ❌ 未实现，TPC-H SF=1 OOM 风险 | **P0 Phase 1 末必做** |
| 6. 可观测性 | ⚠️ 基础存在但指标不够细 | P2 v3.x 逐步增强 |

---

## 建议 v3.0.0 计划调整

### 新增到 Phase 1（Performance Pocket v1）

```
PP-06: Hash Join/Sort 内存限额（3d, P0）
  → 防止 TPC-H SF=1 OOM 的关键任务
  → 验收: SF=1 22/22 可运行无 OOM
```

### 新增到 Phase 2（SQL Completeness）

```
F-06t: Optimizer 规则测试扩展（2d, P1）
F-06p: Planner 逻辑测试扩展（2d, P1）
T-02: 事务状态机压力测试（2d, P0）
```

### 新增到 Phase 3（Infrastructure）

```
I-09ps: COM_MULTI + Prepared Statement 绑定修复（3d, P0）
T-01: Sysbench OLTP 完整适配（3d, P0）
  → 必须跑通才能量化性能达标
```

### 延后至 v3.1.0（不变）

```
S-01: 聚簇索引基础实现（5d）
A-08: 零拷贝缓冲区（3d）
指标增强: /metrics + 性能摘要表（3d）
```

---

*文档版本: 1.0 | 2026-05-06*
*基于 v3.0.0 开发分支源码审计*
