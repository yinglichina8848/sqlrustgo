# v2.9.0 集成测试计划

> **版本**: v2.9.0 (RC)
> **创建日期**: 2026-05-05
> **基于**: TEST_PLAN.md + TEST_REPORT.md 实际结果
> **状态**: 执行中

---

## 一、测试总览

### 1.1 测试分层架构

```
┌─────────────────────────────────────────────┐
│           End-to-End Tests (E2E)             │
│   TPC-H SF=0.1 / SF=1, sysbench, fuzz     │
├─────────────────────────────────────────────┤
│         Integration Tests                    │
│  catalog, planner, optimizer, executor       │
├─────────────────────────────────────────────┤
│           Unit Tests                         │
│  parser, lexer, storage, transaction        │
└─────────────────────────────────────────────┘
```

### 1.2 测试执行统计

| 层级 | 测试数 | 通过 | 失败 | 覆盖率 |
|------|--------|------|------|--------|
| Unit | 4000+ | ~4000 | ~0 | 84.18% |
| Integration | 150+ | 148 | 2 | — |
| E2E | 30+ | 28 | 2 | — |

---

## 二、单元测试详情

### 2.1 测试命令

```bash
# 所有单元测试
cargo test --all-features

# 按 crate 运行
cargo test -p sqlrustgo-parser --all-features
cargo test -p sqlrustgo-executor --all-features
cargo test -p sqlrustgo-storage --all-features
cargo test -p sqlrustgo-transaction --all-features
```

### 2.2 覆盖率目标

| Crate | 覆盖率目标 | 实际 |
|-------|-----------|------|
| executor | ≥60% | 71.08% |
| storage | ≥70% | 75.2% |
| parser | ≥85% | 88.1% |
| transaction | ≥65% | 68.5% |
| **Total** | **≥75%** | **84.18%** |

---

## 三、集成测试详情

### 3.1 Catalog DDL Cache Test

```bash
cargo test --test catalog_ddl_cache_test
```

验证 ALTER TABLE ADD/MODIFY/DROP COLUMN 后 catalog 缓存正确失效。

### 3.2 MVCC Snapshot Isolation Test

```bash
cargo test --test mvcc_snapshot_isolation_test
```

验证并发事务的 MVCC 隔离级别正确性。

### 3.3 Optimizer CBO Accuracy Test

```bash
cargo test --test optimizer_cbo_accuracy_test
```

验证代价模型（CBO）输出正确计划。

### 3.4 Planner Multi-Join Test

```bash
cargo test --test planner_multi_join_test
```

验证多表 JOIN（5 表以上）计划生成。

### 3.5 Network TCP Smoke Test

```bash
cargo test --test network_tcp_smoke_test
```

验证 TCP server/client 基础连接功能。

---

## 四、E2E 测试详情

### 4.1 TPC-H 基准测试

```bash
# SF=0.1 测试（13/22 查询通过）
cargo run --bin bench-cli -- tpch bench --queries all --sf 0.1

# SF=1 测试（9/22 查询通过）
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q4,Q6,Q10,Q13,Q14,Q18,Q20 --sf 1
```

**已验证查询**: Q1, Q2, Q3, Q4, Q5, Q6, Q10, Q13, Q14, Q18, Q20, Q21, Q22

**未验证查询**: Q7, Q8, Q9, Q11, Q12, Q15, Q16, Q17, Q19 (延至 v2.10.0)

### 4.2 SQL Corpus 兼容性测试

```bash
cargo test -p sqlrustgo-parser --test corpus_tests
```

**当前**: 92.6% (449/485 通过)

### 4.3 sysbench OLTP 测试

```bash
# Point select QPS 测试
cargo run --bin bench-cli -- sysbench point_select --threads 4 --time 30

# Range scan 测试
cargo run --bin bench-cli -- sysbench range_scan --threads 4 --time 30
```

**当前**: ~2,000 QPS (目标 ≥10,000，延至 v2.10.0)

---

## 五、混沌 CI 测试

### 5.1 混沌注入测试

```bash
cargo test -p sqlrustgo-executor --test chaos_tests
```

注入延迟、丢包、节点故障，验证系统韧性。

### 5.2 Formal Verification (Phase S)

详见 [PHASE_S_VERIFICATION_WORKFLOW.md](./PHASE_S_VERIFICATION_WORKFLOW.md)

| Proof ID | 验证内容 | 状态 |
|----------|---------|------|
| PROOF-011 | WAL Recovery 完整性 | ✅ PASS |
| PROOF-012 | MVCC 读一致性 | ✅ PASS |
| PROOF-013 | Buffer Pool LRU | ✅ PASS |
| PROOF-014 | Transaction 原子性 | ✅ PASS |
| PROOF-015 | Lock Manager 死锁检测 | ✅ PASS |

---

## 六、测试环境

### 6.1 HP Z6G4 Server

| 组件 | 配置 |
|------|------|
| CPU | Intel Xeon / 40 cores |
| RAM | 256GB |
| OS | Ubuntu 22.04 |
| Rust | 1.94.1 |
| Nomad Runner | nomad-runner |

### 6.2 Nomad CI 配置

Job 定义: `runner.nomad`
Driver: Docker (`sqlrustgo-runner:v1`)

---

## 七、CI/CD 集成

### 7.1 Gitea Actions Workflow

| Workflow | 触发 | Gate |
|----------|------|------|
| `ci.yml` | push/PR | L1 (fast) |
| `gate-ci.yml` | PR to protected | B-Gate |

### 7.2 Coverage Pipeline

| Pipeline | 工具 | 目标 |
|----------|------|------|
| Coverage (Tarpaulin) | cargo-tarpaulin | Line coverage |
| Coverage (LLVM Cov) | cargo-llvm-cov | Branch + path coverage |

---

## 八、已知限制

| 测试项 | 状态 | 说明 |
|--------|------|------|
| TPC-H 18/22 | ❌ 延至 v2.10.0 | 需 JOIN/子查询/视图支持 |
| sysbench QPS ≥10K | ❌ 延至 v2.10.0 | 需 SIMD + 连接池优化 |
| Q17, Q18 | ❌ 延至 v2.10.0 | 相关子查询复杂 |

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
