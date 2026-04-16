# EPIC-v2.5.0 功能完善与集成

**版本**: v2.5.0
**状态**: 待处理
**优先级**: 高
**Epic 负责**: TBD

---

## 概述

根据 v2.5.0 实现分析，部分功能虽然已经实现但未完全集成到执行路径中，或缺少关键测试。需要完善这些功能的集成和测试覆盖。

---

## Issues

### Issue #1501: 索引功能未默认启用

**严重程度**: 中
**状态**: 待处理
**模块**: optimizer / planner / storage

**描述**:
索引功能已完整实现 (B+Tree、Hash、复合索引)，执行器有 `IndexScanVolcanoExecutor`，存储层有 `search_index/range_index` API，但 `planner.rs::should_use_index()` 当前返回 `false`，导致索引路径未被默认启用。

**当前代码位置**:
- 索引实现: `crates/storage/src/bplus_tree/index.rs`
- 执行器: `crates/executor/src/index_scan.rs`
- 入口: `crates/planner/src/planner.rs:791-795` (should_use_index)

**需要完成**:
1. 实现统计信息驱动的 `should_use_index` 逻辑
2. 基于表行数和谓词选择性决定是否使用索引
3. 添加集成测试验证 IndexScanExec 实际执行

**验收标准**:
- [ ] should_use_index 在大表(>1000行) + 高选择性谓词(选择性<20%)时返回 true
- [ ] 集成测试中查询使用索引执行计划
- [ ] EXPLAIN 显示使用 IndexScan 而非 SeqScan

---

### Issue #1502: 存储过程执行未集成

**严重程度**: 高
**状态**: 待处理
**模块**: parser / executor

**描述**:
存储过程解析框架存在 (`parser.rs:3011 parse_create_trigger`)，执行框架也存在 (`executor/src/stored_proc.rs`)，但两者未连接。

**当前代码位置**:
- 解析: `crates/parser/src/parser.rs` (CREATE PROCEDURE 解析)
- 执行: `crates/executor/src/stored_proc.rs`

**需要完成**:
1. 连接解析器到执行器的调用链
2. 实现 CALL 语句的执行路径
3. 添加存储过程集成测试

**验收标准**:
- [ ] CREATE PROCEDURE 可以成功解析
- [ ] CALL proc_name() 可以成功执行
- [ ] 集成测试通过

---

### Issue #1503: 触发器执行未完成

**严重程度**: 高
**状态**: 待处理
**模块**: parser / executor

**描述**:
触发器解析器入口存在 (`parse_create_trigger`)，但触发器的执行引擎未实现。

**当前代码位置**:
- 解析: `crates/parser/src/parser.rs:3692-3746`

**需要完成**:
1. 实现触发器执行引擎
2. 支持 INSERT/UPDATE/DELETE 触发
3. 添加触发器集成测试

**验收标准**:
- [ ] CREATE TRIGGER 成功解析
- [ ] INSERT/UPDATE/DELETE 触发器正确执行
- [ ] 集成测试通过

---

### Issue #1504: SIMD 性能基准测试缺失

**严重程度**: 中
**状态**: 待处理
**模块**: benchmark / executor

**描述**:
SIMD 代码已集成 (`simd_agg` 在 `vectorization.rs`)，但缺少真实的 SIMD vs 非 SIMD 性能对比测试。

**当前代码位置**:
- SIMD: `crates/executor/src/vectorization.rs` (772: pub use simd_agg)
- 调用: `crates/executor/src/parallel_vector_executor.rs`

**需要完成**:
1. 添加 SIMD vs 非 SIMD 性能对比基准
2. 验证 AVX2/AVX-512 实际加速比
3. 条件编译正确性验证

**验收标准**:
- [ ] 有基准测试代码对比 SIMD 和标量执行
- [ ] 实测 SIMD 有 2-4x 加速
- [ ] 在不同 CPU 架构上正确回退

---

### Issue #1505: BloomFilter 与 CBO 联动测试缺失

**严重程度**: 低
**状态**: 待处理
**模块**: optimizer / storage

**描述**:
BloomFilter 已实现并优化 IN/AND 谓词，但未与 CBO 优化器联动测试。

**当前代码位置**:
- BloomFilter: `crates/storage/src/bloom_filter.rs`

**需要完成**:
1. 添加 BloomFilter 与 CBO 联动集成���试
2. 验证 BloomFilter 实际减少 IO

**验收标准**:
- [ ] 测试用例验证 IN 查询使用 BloomFilter
- [ ] 测试用例验证 AND 块跳过
- [ ] 验证 IO 减少

---

### Issue #1506: MVCC GC 压力测试缺失

**严重程度**: 低
**状态**: 待处理
**模块**: transaction / storage

**描述**:
MVCC GC 已实现但缺少长时间压力测试验证。

**当前代码位置**:
- GC: `crates/transaction/src/version_chain.rs`

**需要完成**:
1. 添加高并发场景 GC 稳定性测试
2. 验证长事务场景版本链清理
3. 内存使用稳定性验证

**验收标准**:
- [ ] 72h 压力测试无内存泄漏
- [ ] GC 不阻塞事务
- [ ] 版本链长度受控

---

### Issue #1507: 分布式功能仅框架阶段

**严重程度**: 极高
**状态**: 待处理
**模块**: distributed

**描述**:
分布式功能 (2PC、Raft) 仅在研究/框架阶段，未集成到存储层。

**当前代码位置**:
- 分布锁: `crates/distributed/src/distributed_lock.rs`
- Raft: `crates/distributed/src/raft.rs`

**需要完成**:
1. 确认是否纳入 v3.0 路线图
2. 如果需要：实现 2PC 协议集成
3. 如果不需要：标记为 " Won't Fix" 并说明原因

**验收标准**:
- [ ] 确认规划或标记为 wontfix

---

## 依赖关系

```
#1501 (索引启用)
  └─ 依赖: Issue #1505 (BloomFilter 测试)

#1502 (存储过程)
  └─ 依赖: Issue #1503 (触发器)

#1507 (分布式)
  └─ 前置: v3.0 路线图确认
```

---

## 里程碑

### Phase 1: 核心集成 (v2.5.1)

- [ ] Issue #1501: 索引启用
- [ ] Issue #1502: 存储过程执行
- [ ] Issue #1503: 触发器执行

### Phase 2: 测试完善 (v2.5.2)

- [ ] Issue #1504: SIMD 基准测试
- [ ] Issue #1505: BloomFilter 联动测试
- [ ] Issue #1506: MVCC GC 压力测试

### Phase 3: 规划 (v2.5.3 / v3.0)

- [ ] Issue #1507: 分布式规划

---

## 技术备注

### 索引启用参考代码

```rust
// 建议实现: planner/src/planner.rs
fn should_use_index(&self, table_stats: &TableStats, predicate: &Predicate) -> bool {
    // 大表且低选择性谓词时使用索引
    table_stats.row_count > 1000 && predicate.selectivity() < 0.2
}
```

### 存储过程执行参考

```rust
// 建议实现: executor/src/executor.rs
fn execute_call(&self, proc_name: &str) -> Result<()> {
    let stored_proc = self.catalog.get_stored_proc(proc_name)?;
    self.execute_procedure(stored_proc)
}
```

---

## 资源

- 实现分析报告: `docs/releases/v2.5.0/IMPLEMENTATION_ANALYSIS.md`
- 功能矩阵: `docs/releases/v2.5.0/FEATURE_MATRIX.md`

---

**Epic 状态**: 🟡 进行中
**创建日期**: 2026-04-16
**目标版本**: v2.5.x