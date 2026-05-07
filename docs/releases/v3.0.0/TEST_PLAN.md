# v3.0.0 测试计划

> **版本**: v3.0.0
> **日期**: 2026-05-08
> **状态**: Beta 阶段
> **综合自**: v1.9.0 / v2.7.0 / v2.9.0 历史测试文档 + gate_spec_v300.md

---

## 一、测试阶段总览

| 阶段 | 入口条件 | 核心测试 | 覆盖率目标 | 性能目标 |
|------|----------|----------|-----------|---------|
| **Alpha** | Phase 0-4 开发完成 | L0 冒烟 + L1 模块 | ≥50% | 基线建立 |
| **Beta** | Alpha 通过 + TPC-H SF=0.1 可运行 | L0 + L1 + L2 完整 | ≥75% | TPC-H SF=0.1 22/22 |
| **RC** | Beta 通过 + TPC-H SF=1 可运行 | L0 + L1 + L2 + L3 性能 | ≥85% | TPC-H SF=1 22/22 + QPS 基线 |
| **GA** | RC 通过 + QPS 达标 | 完整 L0~L3 + 混沌 | ≥85% | Point Select ≥10K QPS |

---

## 二、门禁检查项映射

### 2.1 Alpha 门禁 (A-Gate)

| ID | 检查项 | 命令 | 通过标准 |
|----|--------|------|----------|
| A1 | 编译 | `cargo build --all-features --workspace` | 无错误 |
| A2 | 单元测试 | `cargo test --all-features --workspace` | ≥80% 通过 |
| A3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| A4 | 格式 | `cargo fmt --all -- --check` | 无差异 |
| A5 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 无死链 |
| A6 | 覆盖率 | `cargo llvm-cov --all-features` | ≥50% |
| A7 | 安全扫描 | `cargo audit` | 无高危漏洞 |

**Alpha 覆盖率模块级要求**：

| 模块 | 目标 | 模块 | 目标 |
|------|------|------|------|
| executor | ≥45% | optimizer | ≥40% |
| storage | ≥15% | catalog | ≥50% |
| parser | ≥50% | **整体** | **≥50%** |

### 2.2 Beta 门禁 (B-Gate)

| ID | 检查项 | 命令 | 通过标准 |
|----|--------|------|----------|
| B1 | 编译 | `cargo build --release --workspace` | 无错误 |
| B2 | 核心 crate 测试 | L1 核心 crate lib tests | ≥90% 通过 |
| B3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| B4 | 格式 | `cargo fmt --all -- --check` | 无差异 |
| B5 | 覆盖率 | `cargo llvm-cov --all-features` | ≥75% |
| B6 | 安全扫描 | `cargo audit` | 无高危漏洞 |
| B7 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 无死链 |
| B8 | TPC-H SF=0.1 | `bash scripts/gate/check_tpch.sh sf=0.1` | 22/22 |
| B9 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% |

**Beta 稳定性测试 (B-S)**：

| ID | 检查项 | 命令 | 通过标准 |
|----|--------|------|----------|
| B-S1 | concurrency_stress_test | `cargo test --test concurrency_stress_test` | 全部 PASS |
| B-S2 | crash_recovery_test | `cargo test --test crash_recovery_test` | 全部 PASS |
| B-S3 | long_run_stability_test | `cargo test --test long_run_stability_test` | 全部 PASS |
| B-S4 | wal_integration_test | `cargo test --test wal_integration_test` | 全部 PASS |
| B-S5 | network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test` | 全部 PASS |
| B-S10 | SQL operations | `cargo test -p sqlrustgo-sql-corpus test_sql_corpus_operations` | ≥20% (信息项) |

**Beta 覆盖率模块级要求**：

| 模块 | 目标 | 模块 | 目标 |
|------|------|------|------|
| executor | ≥60% | optimizer | ≥50% |
| storage | ≥20% | catalog | ≥60% |
| parser | ≥60% | **整体** | **≥75%** |

### 2.3 RC 门禁 (R-Gate)

| ID | 检查项 | 命令 | 通过标准 |
|----|--------|------|----------|
| R1 | 编译 | `cargo build --release --workspace` | 无错误 |
| R2 | 全量测试 | `cargo test --all-features` | 100% 通过 |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| R4 | 格式 | `cargo fmt --all -- --check` | 无差异 |
| R5 | 覆盖率 | `cargo llvm-cov --all-features` | ≥85% |
| R6 | 安全扫描 | `cargo audit` | 无高危漏洞 |
| R7 | 文档完整性 | check_docs_links.sh + 必选文档 | 无死链/缺失 |
| R8 | SQL Compat | `cargo test -p sqlrustgo-sql-corpus` | ≥95% |
| R9 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh sf=1` | 22/22 无 OOM |
| R10 | 性能基线 | `cargo bench && check_perf_baseline.sh` | QPS 退化 ≤5% |
| R11 | Sysbench | `bash scripts/gate/check_sysbench.sh` | Point/UPDATE/INSERT |
| R12 | MySQL Protocol | mysql:5.7 容器握手 | 连接成功 |

**RC 覆盖率模块级要求**：

| 模块 | 目标 | 模块 | 目标 |
|------|------|------|------|
| executor | ≥75% | optimizer | ≥70% |
| storage | ≥40% | catalog | ≥70% |
| parser | ≥70% | **整体** | **≥85%** |

### 2.4 GA 门禁 (G-Gate)

| ID | 检查项 | 命令 | 通过标准 |
|----|--------|------|----------|
| G1 | 编译 | `cargo build --release --workspace` | 无错误 |
| G2 | 全量测试 | `cargo test --all-features` | 100% 通过 |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| G4 | 格式 | `cargo fmt --all -- --check` | 无差异 |
| G5 | 覆盖率 | `cargo llvm-cov --all-features` | ≥85% |
| G6 | 安全扫描 | `cargo audit` | 无高危漏洞 |
| G7 | Point SELECT QPS | `cargo bench -- point_select` | ≥10,000 ops/s |
| G8 | UPDATE QPS | `cargo bench -- update_simple` | ≥5,000 ops/s |
| G9 | DELETE QPS | `cargo bench -- delete_simple` | ≥2,000 ops/s |
| G10 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh sf=1` | 22/22 无 OOM |
| G11 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥98% |
| G12 | B-S 稳定性 | B-S1~B-S5 | 全部 PASS |
| G13 | MySQL Protocol | mysql:5.7 容器握手 | 连接成功 |

**GA 覆盖率模块级要求**：

| 模块 | 目标 | 模块 | 目标 |
|------|------|------|------|
| executor | ≥80% | optimizer | ≥70% |
| storage | ≥40% | catalog | ≥75% |
| parser | ≥80% | **整体** | **≥85%** |

---

## 三、测试分层体系

### 3.1 L0 冒烟 (<5min)

**目的**: 快速判断分支是否可用

```
L0 检查项
============
[ ] cargo build --release --workspace          # 编译成功
[ ] cargo fmt --all -- --check               # 格式正确
[ ] cargo clippy --all-features -- -D warnings # 无警告
[ ] cargo test -p sqlrustgo-types --lib      # types 冒烟
[ ] cargo test -p sqlrustgo-parser --lib     # parser 冒烟

命令: bash scripts/gate/check_l0_smoke.sh
```

### 3.2 L1 模块回归 (<30min)

**目的**: 验证核心 crate 行为正确

**范围**: 核心 crate lib tests（排除 heavy crates）

```
L1 核心 crates
===============
  - sqlrustgo-types
  - sqlrustgo-parser
  - sqlrustgo-planner
  - sqlrustgo-optimizer
  - sqlrustgo-executor (lib only)
  - sqlrustgo-storage (lib only)
  - sqlrustgo-transaction
  - sqlrustgo-catalog

命令:
cargo test \
  -p sqlrustgo-types \
  -p sqlrustgo-parser \
  -p sqlrustgo-planner \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-executor \
  -p sqlrustgo-storage \
  -p sqlrustgo-transaction \
  -p sqlrustgo-catalog \
  --lib -- --test-threads=8
```

**禁止在 L1 中运行的 heavy crates**:
- sqlrustgo-mysql-server (需要网络监听)
- sqlrustgo-bench (heavy 性能测试)
- sqlrustgo-distributed (需要多节点)
- sqlancer (fuzz 测试)

### 3.3 L2 集成回归 (<90min)

**目的**: 跨模块、跨引擎、跨协议验证

```
L2 范围
========
1. sqlrustgo-sql-corpus (SQL 兼容性)
2. wal_integration_test (WAL + 崩溃恢复)
3. crash_recovery_test (崩溃恢复)
4. concurrency_stress_test (并发压力)
5. optimizer/planner integration tests

命令:
cargo test -p sqlrustgo-sql-corpus -- --test-threads=4
cargo test --test wal_integration_test
cargo test --test crash_recovery_test
cargo test --test concurrency_stress_test
```

### 3.4 L3 深度验证 (>90min / 夜间)

**目的**: 发布级稳定性与性能

```
L3 范围
========
性能测试:
  - TPC-H SF=0.1 / SF=1 (bash scripts/gate/check_tpch.sh)
  - Sysbench (bash scripts/gate/check_sysbench.sh)
  - Point/UPDATE/DELETE QPS (cargo bench)

稳定性测试:
  - long_run_stability_test (长时间运行)
  - network_tcp_smoke_test (网络稳定性)

覆盖率:
  - cargo llvm-cov --all-features --json (完整覆盖率)

混沌工程:
  - crash_injection_test (崩溃注入)
  - chaos_test (混沌测试)
```

---

## 四、SQL Corpus 测试

### 4.1 测试结构

```
sqlrustgo-sql-corpus 测试分类
==============================
test_sql_corpus_all        # 全部 SQL 语法（485 cases）
test_sql_corpus_operations # DML 操作（55 cases） ← 当前最弱
test_sql_corpus_aggregates # 聚合函数
test_sql_corpus_joins     # JOIN 变体
test_sql_corpus_subqueries # 子查询
```

### 4.2 operations 类别弱项分析

**当前状态**: 20% (11/55) 通过，44 个语法不支持

| 不支持类别 | 数量 | 优先级 |
|-----------|------|--------|
| BACKUP | 1 | P3 |
| SAVEPOINT | 1 | P3 |
| SET TRANSACTION ISOLATION LEVEL | 1 | P2 |
| LIMIT/OFFSET | 1 | P1 |
| TRUNCATE | 1 | P2 |
| REPLACE | 1 | P2 |
| SHOW | ~10 | P2 |
| EXPLAIN ANALYZE | 1 | P1 |
| TEMPORARY TABLE | 1 | P2 |
| ALTER TABLE INPLACE | ~5 | P1 |
| BATCH INSERT | ~10 | P2 |

**补测优先级**:
- **P1**: LIMIT/OFFSET, ALTER TABLE INPLACE, EXPLAIN ANALYZE
- **P2**: REPLACE, SHOW, TEMPORARY TABLE, TRUNCATE, BATCH INSERT
- **P3**: BACKUP, SAVEPOINT

### 4.3 覆盖率追踪命令

```bash
# 全部 SQL 语法
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_all -- --nocapture

# 分类追踪
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_operations -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_aggregates -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_joins -- --nocapture
cargo test -p sqlrustgo-sql-corpus test_sql_corpus_subqueries -- --nocapture
```

---

## 五、TPC-H 测试

### 5.1 测试分级

| 级别 | 触发条件 | 数据规模 | 执行时间 | 要求 |
|------|----------|----------|----------|------|
| **SF=0.1** | PR / Beta | ~100KB | <30s | 22/22 |
| **SF=1** | RC / 发布前 | ~1MB | <5min | 22/22 无 OOM |
| **SF=10** | 夜间 | ~10MB | <30min | 22/22 记录 p99 |
| **SF=100** | 季度 | ~100MB | >1h | 可选 |

### 5.2 执行命令

```bash
# SF=0.1 (Beta 门禁)
bash scripts/gate/check_tpch.sh sf=0.1

# SF=1 (RC/GA 门禁)
bash scripts/gate/check_tpch.sh sf=1

# 详细输出
bash scripts/gate/check_tpch.sh sf=0.1 --verbose
```

### 5.3 风险分级

| 风险 | Query | 问题 |
|------|-------|------|
| 🔴 高 | Q17, Q18 | 容易 OOM / 超时 |
| 🟡 中 | Q1, Q3, Q6, Q9 | 执行时间不稳定 |
| 🟢 低 | Q2, Q4, Q5, Q7-Q8, Q10-Q16, Q19-Q22 | 稳定通过 |

---

## 六、性能测试

### 6.1 QPS 基线目标

| 指标 | Alpha 基线 | Beta 目标 | RC/GA 目标 |
|------|-----------|-----------|------------|
| Point SELECT QPS | ≥7,000 | ≥10,000 | ≥10,000 |
| UPDATE QPS | ≥40,000 | ≥5,000 | ≥5,000 |
| DELETE QPS | ≥60,000 | ≥2,000 | ≥2,000 |

### 6.2 回归判定

```
QPS 退化 ≤5%   → PASS
QPS 退化 5-20% → 需人工解释
QPS 退化 >20%  → FAIL
```

### 6.3 执行命令

```bash
# 性能回归检查
bash scripts/gate/check_regression.sh

# Sysbench 检查
bash scripts/gate/check_sysbench.sh

# 详细 QPS 测量
cargo bench -- --measure-throughput
```

---

## 七、覆盖率弱项分析

### 7.1 模块级覆盖率（v2.9.0 基线数据）

| 模块 | 当前覆盖 | Beta 目标 | RC/GA 目标 | 差距 | 风险 |
|------|---------|-----------|------------|------|------|
| optimizer | 0% | 50% | 70% | 70% | 🔴 CRITICAL |
| planner | <1% | 50% | 70% | ~69% | 🔴 CRITICAL |
| transaction | ~0% | 50% | 70% | ~70% | 🔴 CRITICAL |
| types | 4% | 50% | 70% | 66% | 🔴 CRITICAL |
| storage | 1% | 20% | 40% | 39% | 🟡 HIGH |
| catalog | 1% | 60% | 75% | 74% | 🔴 CRITICAL |
| parser | 20% | 60% | 80% | 60% | 🟡 HIGH |
| executor | 72% | 60% | 80% | -12% | 🟢 LOW |

> **数据来源**: v2.9.0 TEST_STATUS_20260503.md

### 7.2 补测优先级决策树

```
发现模块覆盖率低于目标
         │
         ├── 差距 <10% 且非核心路径
         │         → 标记 LOW，忽略或下版本处理
         │
         ├── 差距 10-30% 或核心路径
         │         → 标记 HIGH，优先补测
         │         → 补测：边界条件 + 错误路径
         │
         └── 差距 >30% 或模块 = 0%
                  → 标记 CRITICAL，必须补测
                  → 补测：主路径 + 边缘路径
                  → 第一步：建立最小测试集（10-20 tests）
```

### 7.3 各模块缺失场景

| 模块 | 缺失的高风险场景 |
|------|----------------|
| optimizer | 物理计划生成、JOIN 顺序优化、代价模型计算、LIMIT 下推 |
| planner | 逻辑计划解析、子查询去嵌套、GROUP BY 聚合折叠 |
| storage | B+Tree 并发插入、页缓存淘汰策略、WAL 写入路径、崩溃恢复 |
| transaction | MVCC 可见性判断、死锁检测与回滚、Savepoint 嵌套、RC/RR 差异 |
| types | Value 类型转换、边界条件、NULL 语义 |
| parser | DDL 语法、复杂表达式、信息模式查询 |
| catalog | Schema 操作、DDL 事务性 |

---

## 八、执行节奏

### 8.1 日常开发

```
每次 commit/PR:
  L0 冒烟: <5min  (自动 CI)

PR 合并前:
  L0 + L1: <30min (必须通过)
```

### 8.2 发布节奏

| 阶段 | 执行内容 | 时间 |
|------|----------|------|
| Beta 前 | L0 + L1 + L2 + 关键 L3 | <3h |
| RC 前 | L0 + L1 + L2 + 完整 L3 | <6h + 夜间 |
| GA 前 | 完整 L0~L3 + 混沌 | <24h |

### 8.3 覆盖率趋势追踪

```bash
# 每次 L3 执行时记录
cargo llvm-cov --all-features --json --output-path /tmp/cov.json

# 提取数据到趋势文件
echo "$(date -u +%Y-%m-%d),$total_cov,$optimizer_cov,$planner_cov" \
  >> docs/releases/v3.0.0/coverage_trend.csv
```

---

## 九、已知问题与风险

### 9.1 当前风险项

| Issue | 描述 | 阶段 | 状态 |
|-------|------|------|------|
| #451 | SQL operations 20% (11/55) | Beta | OPEN |
| #392 | CBO 代价模型集成 | Beta | OPEN |
| #379 | 事务状态机压力测试 | Beta | OPEN |
| #380 | Optimizer 测试扩展 | RC | OPEN |
| #381 | Planner 测试扩展 | RC | OPEN |

### 9.2 测试执行风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| B2 `cargo test --all-features` 超时 | Beta 门禁无法完成 | 已改为 L1 核心 crate 测试 |
| B5 `cargo llvm-cov --all-features` 超时 | 覆盖率无法测量 | 已优化为只测核心 crates |
| TPC-H Q1 性能退化 | Beta 门禁 FAIL | 需要优化 Q1 执行路径 |

---

## 十、相关文档

| 文档 | 作用 |
|------|------|
| `gate_spec_v300.md` | 门禁定义 SSOT |
| `gate_lifecycle_tracking.md` | 门禁失败项追踪 |
| `docs/governance/TEST_PLAN_v3.md` | 详细测试计划（含弱项分析） |
| `docs/governance/governance_self_improvement.md` | 治理自我进化机制 |
| `scripts/gate/check_beta_v300.sh` | Beta 门禁脚本 |
| `scripts/gate/check_tpch.sh` | TPC-H 测试脚本 |
| `scripts/gate/check_l0_smoke.sh` | L0 冒烟脚本 |

---

*本文档综合自 v1.9.0 / v2.7.0 / v2.9.0 / v3.0.0 历史测试文档*
*最后更新: 2026-05-08*
