# v3.1.0 测试覆盖率系统提升计划

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **状态**: 🟡 规划中  
> **目标**: 将整体覆盖率从 ~50% 提升至 ≥65%（Beta）/ ≥75%（GA），关键模块达到 80%+

---

## 一、现状诊断

### 1.1 覆盖率数据（v3.0.0 GA）

| crate | 行覆盖率 | 函数覆盖率 | 差距分析 |
|--------|---------|-----------|---------|
| `sqlrustgo-parser` | 62.97% | ~60% | DDL/DCL 覆盖不足 |
| `sqlrustgo-executor` | ~50% | ~50% | Volcano/H Volcano 路径覆盖不足 |
| `sqlrustgo-planner` | ~45% | ~40% | CBO/join reorder 未测 |
| `sqlrustgo-optimizer` | ~30% | ~25% | CostModel 完全未覆盖 |
| `sqlrustgo-storage` | ~55% | ~50% | B+Tree 崩溃路径未测 |
| `sqlrustgo-transaction` | ~50% | ~45% | SSI/Gap Lock 路径未测 |
| `sqlrustgo-catalog` | ~60% | ~55% | System table 覆盖不足 |
| `sqlrustgo-network` | ~40% | ~35% | MySQL protocol 握手覆盖不足 |
| **总体** | **~50%** | **~45%** | |

### 1.2 覆盖缺口根因

| 根因 | 影响 crate | 具体缺口 |
|------|-----------|---------|
| C1: DDL 测试不足 | parser, catalog | CREATE/ALTER/DROP 覆盖率 < 30% |
| C2: 混沌/崩溃路径未测 | storage, transaction | 崩溃注入测试存在但被 #[ignore] |
| C3: CBO/CostModel 完全无测试 | optimizer, planner | CostModel 0% 覆盖 |
| C4: MySQL Protocol 握手未覆盖 | network, mysql-server | 连接建立/认证 0% |
| C5: 复杂查询路径未覆盖 | executor | Hash Join/Merge Join null-handling 差 |
| C6: 并发压测覆盖不足 | transaction | 死锁/SSI 并发 0% |

---

## 二、覆盖率提升目标

### 2.1 分阶段目标

| 阶段 | 截止日期 | 总体覆盖 | 关键模块覆盖 |
|------|---------|---------|-------------|
| **Alpha** | 2026-06-01 | ≥40% | parser ≥65%, executor ≥55% |
| **Beta** | 2026-07-01 | ≥55% | planner ≥50%, optimizer ≥40% |
| **RC** | 2026-08-01 | ≥65% | transaction ≥60%, storage ≥65% |
| **GA** | 2026-09-01 | ≥75% | optimizer ≥50%, planner ≥60% |

### 2.2 逐 crate 目标

| crate | Beta | RC | GA |
|-------|------|----|----|
| parser | 65% | 70% | **75%** |
| executor | 55% | 65% | **70%** |
| planner | 50% | 55% | **60%** |
| optimizer | 40% | 50% | **60%** |
| storage | 55% | 60% | **70%** |
| transaction | 50% | 60% | **70%** |
| catalog | 60% | 65% | **70%** |
| network | 45% | 55% | **60%** |
| mysql-server | 40% | 50% | **55%** |
| **总体** | **50%** | **60%** | **65%** |

---

## 三、测试覆盖增强任务

### 3.1 C1: DDL 测试覆盖（parser + catalog）

**当前状态**: DDL 测试缺失严重

**新增测试**:

```bash
# parser DDL 测试
tests/ddl/
├── create_table_test.rs      # CREATE TABLE 完整变体
├── alter_table_test.rs       # ALTER TABLE ADD/DROP/RENAME COLUMN
├── drop_table_test.rs       # DROP TABLE CASCADE/RESTRICT
├── create_index_test.rs     # CREATE INDEX / DROP INDEX
├── create_view_test.rs      # CREATE VIEW
└── ddl_grammar_test.rs     # 完整 DDL 语法解析

# catalog DDL 集成测试
tests/catalog_ddl_integration_test.rs
```

**覆盖目标**:
- CREATE TABLE: 基本、NOT NULL、DEFAULT、AUTO_INCREMENT、PRIMARY KEY、UNIQUE、FOREIGN KEY
- ALTER TABLE: ADD/DROP/MODIFY COLUMN、RENAME、ADD INDEX
- DROP TABLE: CASCADE、RESTRICT、IF EXISTS

### 3.2 C2: 崩溃路径覆盖（storage + transaction）

**当前问题**: chaos_crash_* 测试存在但部分 #[ignore]

**新增/修复测试**:

```bash
# storage 崩溃测试
crates/storage/tests/
├── crash_wal_before_test.rs      # BP2-2-S1
├── crash_wal_after_test.rs       # BP2-2-S2
├── crash_precommit_test.rs        # BP2-2-S3
├── crash_checkpoint_test.rs      # BP2-2-S4
└── crash_torn_page_test.rs       # BP2-2-S5

# transaction 崩溃测试
crates/transaction/tests/
├── crash_recovery_wal_test.rs    # WAL replay 一致性
├── crash_mvcc_visibility_test.rs # MVCC 崩溃可见性
└── crash_binlog_test.rs          # binlog 崩溃安全
```

**关键**: 所有 chaos_crash_* 测试必须移除 #[ignore]，自动化运行

### 3.3 C3: CBO/CostModel 测试（optimizer + planner）

**当前状态**: CostModel 0% 覆盖

**新增测试**:

```bash
# optimizer cost model 测试
crates/optimizer/tests/
├── cost_model_test.rs            # 基础代价计算
├── index_selection_test.rs       # 索引选择代价对比
├── join_order_cost_test.rs       # Join 顺序代价
└── access_path_cost_test.rs      # 访问路径代价

# planner CBO 集成测试
tests/cbo_cost_integration_test.rs  # 重构自 cbo_integration_test
```

**测试场景**:

```rust
// cost_model_test.rs
#[test]
fn test_seq_scan_vs_index_scan() {
    // 全表扫描代价 vs 索引扫描代价
    let seq_cost = cost_model.seq_scan_cost(table_stats);
    let idx_cost = cost_model.index_scan_cost(table_stats, index_stats);
    assert!(idx_cost < seq_cost); // 索引代价应更低
}

#[test]
fn test_join_order_cost() {
    // 小表先驱动代价更低
    let cost_abc = cost_model.join_order_cost([table_a, table_b, table_c]);
    let cost_cba = cost_model.join_order_cost([table_c, table_b, table_a]);
    assert_eq!(cost_abc, cost_cba); // 代价应相同
}
```

### 3.4 C4: MySQL Protocol 握手测试（network + mysql-server）

**当前状态**: 握手/认证 0% 覆盖

**新增测试**:

```bash
# MySQL Protocol 测试
crates/mysql-server/tests/
├── handshake_test.rs           # 握手/认证流程
├── auth_packet_test.rs        # 认证包解析
├── connection_lifecycle_test.rs # 连接建立/复用/关闭
└── capability_handshake_test.rs # 能力位协商

crates/network/tests/
├── tcp_handshake_test.rs      # TCP 层握手
├── tls_handshake_test.rs      # TLS 握手（已有部分）
└── protocol_switch_test.rs    # 协议切换
```

### 3.5 C5: 复杂查询路径覆盖（executor）

**当前缺口**: Hash Join null-handling、Merge Join、复杂表达式

**新增测试**:

```bash
# executor 复杂路径测试
crates/executor/tests/
├── hash_join_null_test.rs        # Hash Join null 值处理
├── merge_join_test.rs            # Merge Join 实现
├── nested_loop_join_test.rs      # NLJ 优化路径
├── expression_eval_test.rs        # 复杂表达式求值
│   ├── case_when_test.rs        # CASE WHEN
│   ├── coalesce_test.rs         # COALESCE
│   └── cast_complex_test.rs     # CAST 复杂类型
└── aggregation_null_test.rs      # COUNT(DISTINCT)、GROUP BY NULL
```

### 3.6 C6: 并发压测覆盖（transaction）

**当前缺口**: SSI/Gap Lock 并发、死锁检测

**新增测试**:

```bash
# transaction 并发测试
crates/transaction/tests/
├── ssi_concurrent_test.rs        # SSI 并发写入偏序
├── gap_lock_concurrent_test.rs   # Gap Lock 并发插入
├── deadlock_detection_test.rs    # 死锁检测延迟
├── tx_isolation_levels_test.rs   # 4 种隔离级别验证
│   ├── read_uncommitted_test.rs
│   ├── read_committed_test.rs
│   ├── repeatable_read_test.rs
│   └── serializable_test.rs
└── mvcc_concurrent_test.rs       # MVCC 并发可见性
```

---

## 四、测试框架增强

### 4.1 自动化覆盖率收集

```bash
# 每 PR 自动收集覆盖率，差距 > 5% 阻断合并
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

### 4.2 覆盖率门禁脚本

```bash
# scripts/gate/check_coverage_v310.sh
# 按 crate 检查覆盖率，未达标阻断

for crate in parser executor planner optimizer storage transaction catalog network mysql-server; do
    threshold=$(get_threshold $crate $GATE_PHASE)
    actual=$(cargo llvm-cov -p sqlrustgo-$crate --summary-only | grep TOTAL | awk '{print $NF}' | tr -d '%')
    if (( $(echo "$actual < $threshold" | bc -l) )); then
        echo "FAIL: sqlrustgo-$crate $actual% < $threshold%"
        exit 1
    fi
done
```

### 4.3 覆盖率趋势追踪

| 指标 | Alpha 目标 | Beta 目标 | RC 目标 | GA 目标 |
|------|-----------|-----------|---------|---------|
| parser | 62% | 65% | 70% | **75%** |
| executor | 45% | 55% | 65% | **70%** |
| planner | 35% | 50% | 55% | **60%** |
| optimizer | 20% | 40% | 50% | **60%** |
| storage | 45% | 55% | 60% | **70%** |
| transaction | 40% | 50% | 60% | **70%** |
| **总体** | **40%** | **50%** | **60%** | **65%** |

---

## 五、测试覆盖率验证机制

### 5.1 覆盖率差距报警

```bash
# 在 check_coverage.sh 中集成
# 覆盖率差距 > 5% 触发警告，> 10% 阻断

PREV_COVERAGE=$(cat .coverage_history/v3.1.0-alpha.txt 2>/dev/null || echo "0")
CURRENT_COVERAGE=$(cargo llvm-cov --all-features --lib --summary-only | grep TOTAL | awk '{print $NF}' | tr -d '%')

REGRESSION=$(python3 -c "print(max(0, $PREV_COVERAGE - $CURRENT_COVERAGE))")

if (( $(echo "$REGRESSION > 10" | bc -l) )); then
    echo "🔴 BLOCK: Coverage regression $REGRESSION% (threshold: 10%)"
    exit 1
elif (( $(echo "$REGRESSION > 5" | bc -l) )); then
    echo "🟡 WARNING: Coverage regression $REGRESSION%"
fi
```

### 5.2 覆盖率报告生成

```bash
# 每次 RC/GA Gate 生成 HTML 覆盖率报告
cargo llvm-cov --html --open  # 生成覆盖率可视化
cargo llvm-cov --lcov > coverage.lcov  # 导入 CI
```

---

## 六、关键测试清单（按 crate）

### parser

| 测试文件 | 覆盖目标 | 优先级 |
|---------|---------|--------|
| `tests/ddl/create_table_test.rs` | CREATE TABLE 100% | P0 |
| `tests/ddl/alter_table_test.rs` | ALTER TABLE 100% | P0 |
| `tests/ddl/drop_table_test.rs` | DROP TABLE 100% | P1 |
| `tests/expression_case_test.rs` | CASE WHEN 100% | P1 |
| `tests/subquery_test.rs` | 子查询 100% | P1 |

### executor

| 测试文件 | 覆盖目标 | 优先级 |
|---------|---------|--------|
| `crates/executor/tests/hash_join_null_test.rs` | Hash Join null 100% | P0 |
| `crates/executor/tests/merge_join_test.rs` | Merge Join 100% | P0 |
| `tests/aggregate_null_test.rs` | 聚合 null 100% | P1 |
| `tests/volcano_model_test.rs` | Volcano 100% | P2 |

### optimizer

| 测试文件 | 覆盖目标 | 优先级 |
|---------|---------|--------|
| `crates/optimizer/tests/cost_model_test.rs` | CostModel 100% | P0 |
| `crates/optimizer/tests/index_selection_test.rs` | 索引选择 100% | P0 |
| `crates/optimizer/tests/join_order_cost_test.rs` | Join 代价 100% | P1 |

### storage

| 测试文件 | 覆盖目标 | 优先级 |
|---------|---------|--------|
| `crates/storage/tests/crash_wal_*.rs` | 崩溃路径 100% | P0 |
| `crates/storage/tests/bplus_tree_test.rs` | B+Tree 100% | P1 |
| `tests/page_eviction_test.rs` | 页面淘汰 100% | P2 |

### transaction

| 测试文件 | 覆盖目标 | 优先级 |
|---------|---------|--------|
| `crates/transaction/tests/ssi_concurrent_test.rs` | SSI 并发 100% | P0 |
| `crates/transaction/tests/gap_lock_concurrent_test.rs` | Gap Lock 100% | P0 |
| `crates/transaction/tests/deadlock_detection_test.rs` | 死锁检测 100% | P0 |

---

## 七、执行计划

### 7.1 Alpha 阶段（2026-06-01 前）

```
Week 1-2:
  - 新增 DDL 测试 (create_table, alter_table, drop_table)
  - 修复 chaos_crash_* 测试 #[ignore] 状态
  - 覆盖率基线建立

Week 3-4:
  - Hash Join null 测试
  - Merge Join 测试
  - cost_model_test.rs
```

### 7.2 Beta 阶段（2026-07-01 前）

```
Week 5-6:
  - SSI/Gap Lock 并发测试
  - planner CBO 测试
  - optimizer cost model 测试

Week 7-8:
  - MySQL handshake 测试
  - catalog system table 测试
```

### 7.3 RC 阶段（2026-08-01 前）

```
Week 9-12:
  - 全部 chaos_crash 测试
  - 表达式求值覆盖补全
  - network protocol 测试
```

---

*本文档由 hermes agent 创建。*
*每次 coverage 检查后更新状态。*
